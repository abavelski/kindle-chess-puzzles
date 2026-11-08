"""Private identifiers never enter the source tree or routine output."""
import importlib.util
import json
from pathlib import Path
import stat
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
class ConfigTests(unittest.TestCase):
    def test_prepare_from_private_probe_record_without_printing_identifiers(self):
        spec=importlib.util.spec_from_file_location('config',ROOT/'tools/prepare_chessnut_config.py')
        module=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as folder:
            source=Path(folder)/'devices.private.json'
            target=Path(folder)/'runtime.private.json'
            source.write_text(json.dumps({'ble':{'address':'00:11:22:33:44:55'},'kindle_ssh':{'host':'private'}}))
            module.prepare(source,target)
            self.assertEqual(json.loads(target.read_text()),{'address':'00:11:22:33:44:55'})
            self.assertEqual(stat.S_IMODE(target.stat().st_mode),0o600)
            source.write_text('{"ble":{"address":"invalid"}}')
            original=target.read_bytes()
            with self.assertRaises(ValueError): module.prepare(source,target)
            self.assertEqual(target.read_bytes(),original)
