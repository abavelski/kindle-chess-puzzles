"""The selector must fail closed until each app has a distinct binary and installer."""
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]


def load_tool():
    spec = importlib.util.spec_from_file_location('kindle_build', ROOT / 'tools/kindle_build.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AppCliTests(unittest.TestCase):
    def test_default_and_explicit_puzzles_dispatch_to_existing_pipeline(self):
        tool = load_tool()
        for argv in (['kindle_build.py', 'build'],
                     ['kindle_build.py', 'build', '--app', 'puzzles']):
            with self.subTest(argv=argv), mock.patch.object(tool, 'build') as build:
                with mock.patch.object(sys, 'argv', argv):
                    tool.main()
                build.assert_called_once_with()

    def test_review_rejected_before_any_build_stage_or_deploy_side_effect(self):
        tool = load_tool()
        for action in ('build', 'stage', 'deploy'):
            with self.subTest(action=action):
                with mock.patch.object(sys, 'argv',
                                       ['kindle_build.py', action, '--app', 'review']):
                    with mock.patch.object(tool, 'build') as build, \
                         mock.patch.object(tool, 'stage') as stage, \
                         mock.patch.object(tool, 'deploy') as deploy, \
                         mock.patch('sys.stderr'):
                        with self.assertRaises(SystemExit) as result:
                            tool.main()
                        self.assertEqual(result.exception.code, 2)
                        build.assert_not_called()
                        stage.assert_not_called()
                        deploy.assert_not_called()

    def test_unknown_app_fails_validation(self):
        tool = load_tool()
        with mock.patch.object(sys, 'argv',
                               ['kindle_build.py', 'build', '--app', 'unknown']), \
             mock.patch('sys.stderr'):
            with self.assertRaises(SystemExit) as result:
                tool.main()
            self.assertEqual(result.exception.code, 2)


if __name__ == '__main__':
    unittest.main()
