import tempfile
import unittest
from pathlib import Path

from tools import clean_pgn


class CleanPgnTests(unittest.TestCase):
    def test_unwraps_multiline_pre_comments_without_changing_other_pgn(self):
        source = '[Event "[%pre Keep header]"]\n1. e4 { [%pre Заголовок\nТекст] [%cal Ge2e4] } (1. d4 { [%pre Или] }) *\n'
        expected = '[Event "[%pre Keep header]"]\n1. e4 { Заголовок\nТекст  } (1. d4 { Или }) *\n'
        self.assertEqual(clean_pgn.clean_text(source), (expected, 3))
        self.assertEqual(clean_pgn.clean_text(expected), (expected, 0))

    def test_semicolon_comments_and_unknown_directives(self):
        self.assertEqual(
            clean_pgn.clean_text('1. e4 ; [%pre Note] [%eval 0.5]\n*\n'),
            ('1. e4 ; Note [%eval 0.5]\n*\n', 1),
        )

    def test_malformed_pre_wrapper_fails(self):
        with self.assertRaisesRegex(ValueError, 'Unclosed'):
            clean_pgn.clean_text('1. e4 { [%pre Keep this } *')

    def test_removes_board_directives_only_from_comments(self):
        source = '[Event "[%cal Ga4c4]"]\n1. e4 { Текст [%csl Ga7,Gc4] [%cal Ga4c4,Ga4a7] дальше [%eval 0.5] } ; [%cal Ge2e4]\n*\n'
        expected = '[Event "[%cal Ga4c4]"]\n1. e4 { Текст   дальше [%eval 0.5] } ; \n*\n'
        self.assertEqual(clean_pgn.clean_text(source), (expected, 3))
        self.assertEqual(clean_pgn.clean_text(expected), (expected, 0))

    def test_incomplete_board_directive_fails(self):
        for directive in ('csl', 'cal'):
            with self.subTest(directive=directive), self.assertRaisesRegex(ValueError, 'Unclosed'):
                clean_pgn.clean_text(f'1. e4 {{ [%{directive} Ga7 }} *')

    def test_cli_preserves_source_and_existing_output_on_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'game.pgn'
            output = Path(tmp) / 'clean.pgn'
            source.write_text('1. e4 { [%pre Текст] } *', encoding='utf-8')
            original = source.read_bytes()
            self.assertEqual(clean_pgn.main([str(source), '-o', str(output)]), 0)
            self.assertEqual(source.read_bytes(), original)
            self.assertEqual(output.read_text(), '1. e4 { Текст } *')
            previous = output.read_bytes()
            source.write_text('1. e4 { [%pre Broken } *', encoding='utf-8')
            self.assertEqual(clean_pgn.main([str(source), '-o', str(output)]), 2)
            self.assertEqual(output.read_bytes(), previous)


if __name__ == '__main__':
    unittest.main()
