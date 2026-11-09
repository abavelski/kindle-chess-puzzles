"""The selector binds a real product; unfinished deployment must fail closed."""
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

    def test_review_build_dispatches_to_real_product(self):
        tool = load_tool()
        with mock.patch.object(sys, 'argv', ['kindle_build.py', 'build', '--app', 'review']), mock.patch.object(tool, 'build') as build:
            tool.main()
            build.assert_called_once_with('review')

    def test_play_build_dispatches_to_real_product(self):
        tool = load_tool()
        with mock.patch.object(sys, 'argv', ['kindle_build.py', 'build', '--app', 'play']), mock.patch.object(tool, 'build') as build:
            tool.main()
            build.assert_called_once_with('play')

    def test_review_deploy_dispatches_only_to_selected_product(self):
        tool = load_tool()
        with mock.patch.object(sys, 'argv', ['kindle_build.py', 'deploy', '--app', 'review', '--host', 'root@device']), mock.patch.object(tool, 'deploy') as deploy:
            tool.main()
            self.assertEqual(deploy.call_args.kwargs, {'app': 'review'})
            self.assertEqual(deploy.call_args.args[0], tool.output_dir('review') / 'stage')

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
