import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('study_report', Path(__file__).parents[1] / 'scripts/study_report.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'route.json').write_text('{"requests": 12}')
        self.data = {
            'version': 1, 'title': 'Service route study', 'status': 'complete',
            'scope': 'Verify requests through the declared route', 'source_revision': 'fixture-rev',
            'next_experiment': 'No remaining experiment in this fixture',
            'trace_limitations': ['No browser recording'], 'assumptions': [], 'blockers': [],
            'artifacts': [{'id': 'route', 'path': 'route.json', 'label': 'Route evidence',
                           'sha256': hashlib.sha256((self.root / 'route.json').read_bytes()).hexdigest()}],
            'experiments': [],
            'checks': [{'name': 'Request count', 'status': 'pass', 'expected': 12, 'measured': 12,
                        'unit': 'requests', 'tolerance': {'absolute': 0}, 'evidence': ['route']}],
        }
        self.manifest = self.root / 'study.json'

    def save(self):
        self.manifest.write_text(json.dumps(self.data))

    def test_non_cad_manifest_renders_and_retains_evidence_links(self):
        self.save()
        report.render(self.manifest, [self.root / 'review.html', self.root / 'draft.md'])
        self.assertIn('href="route.json"', (self.root / 'review.html').read_text())
        self.assertIn('12 requests', (self.root / 'draft.md').read_text())
        self.assertEqual((self.root / 'route.json').read_text(), '{"requests": 12}')

    def test_inaccurate_numerical_success_is_rejected(self):
        self.data['checks'][0]['measured'] = 11
        self.save()
        with self.assertRaisesRegex(ValueError, 'disagrees with tolerance'):
            report.validate(self.manifest)

    def test_reopen_float_drift_obeys_relative_tolerance(self):
        self.data['checks'][0].update(expected=7579.35646328853, measured=7579.356463288528,
                                      unit='mm3', tolerance={'relative': .0001})
        self.save()
        self.assertEqual(report.validate(self.manifest)['status'], 'complete')

    def test_unsupported_check_prevents_completion_but_allows_review(self):
        self.data['checks'].append({'name': 'Topology', 'status': 'unsupported', 'evidence': ['route']})
        self.save()
        with self.assertRaisesRegex(ValueError, 'complete requires'):
            report.validate(self.manifest)
        self.data['status'] = 'awaiting_review'
        self.save()
        self.assertEqual(report.validate(self.manifest)['checks'][1]['status'], 'unsupported')

    def test_hash_drift_and_missing_evidence_are_detected(self):
        self.save()
        (self.root / 'route.json').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            report.validate(self.manifest)
        (self.root / 'route.json').unlink()
        with self.assertRaisesRegex(ValueError, 'missing artifact'):
            report.validate(self.manifest)

    def test_outside_and_symlink_artifacts_are_rejected(self):
        for filename in ('../outside', str(self.root.parent / 'outside')):
            self.data['artifacts'][0]['path'] = filename
            self.save()
            with self.assertRaisesRegex(ValueError, 'escapes'):
                report.validate(self.manifest)
        (self.root / 'external').symlink_to(self.root.parent)
        self.data['artifacts'][0]['path'] = 'external/anything'
        self.save()
        with self.assertRaisesRegex(ValueError, 'escapes'):
            report.validate(self.manifest)

    def test_render_escapes_text_and_protects_originals(self):
        self.data['title'] = '<script>alert(1)</script>'
        self.save()
        review, draft = self.root / 'review.html', self.root / 'draft.md'
        draft.write_text('authored history')
        with self.assertRaisesRegex(ValueError, 'output exists'):
            report.render(self.manifest, [review, draft])
        self.assertFalse(review.exists())
        self.assertEqual(draft.read_text(), 'authored history')
        with self.assertRaisesRegex(ValueError, 'cannot overwrite'):
            report.render(self.manifest, [review, self.root / 'route.json'], replace=True)
        report.render(self.manifest, [review, self.root / 'new-draft.md'])
        self.assertNotIn('<script>', review.read_text())
        self.assertIn('&lt;script&gt;', review.read_text())

    def test_failures_remain_reportable_and_evidence_ids_are_checked(self):
        self.data['checks'].append({'name': 'Prior failure', 'status': 'fail', 'expected': 12, 'measured': 0,
                                   'unit': 'requests', 'tolerance': {'absolute': 0},
                                   'required_for_acceptance': False, 'evidence': ['route']})
        self.save()
        self.assertEqual(report.validate(self.manifest)['checks'][1]['status'], 'fail')
        self.data['checks'][1]['evidence'] = ['invented']
        self.save()
        with self.assertRaisesRegex(ValueError, 'unknown evidence'):
            report.validate(self.manifest)


if __name__ == '__main__':
    unittest.main()
