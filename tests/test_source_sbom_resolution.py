"""Prevent manifest ranges from masquerading as installed package versions."""

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("source_sbom", ROOT / "scripts/generate_source_sbom.py")
sbom = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sbom)


class SourceInventoryBoundaries(unittest.TestCase):
    def test_requirements_have_no_resolved_package_identity(self):
        document = sbom.generate()
        requirements = [item for item in document["components"] if item["bom-ref"].startswith("cargo-requirement:")]
        self.assertTrue(requirements)
        for item in requirements:
            with self.subTest(name=item["name"]):
                self.assertNotIn("version", item)
                self.assertNotIn("purl", item)
                self.assertTrue(any(prop["name"] == "searchright:version-requirement" for prop in item["properties"]))
        properties = {item["name"]: item["value"] for item in document["metadata"]["properties"]}
        self.assertEqual(properties["searchright:evidence-level"], "manifest-and-automation-inventory")
        self.assertIn("neither inventory proves binary composition", properties["searchright:claim-boundary"])

    def test_workspace_license_inheritance_produces_strings(self):
        document = sbom.generate()
        licenses = [license["expression"] for item in document["components"] for license in item.get("licenses", [])]
        self.assertTrue(licenses)
        self.assertTrue(all(isinstance(value, str) and value for value in licenses))

    def test_dependency_refs_stay_within_inventory(self):
        document = sbom.generate()
        refs = {item["bom-ref"] for item in document["components"]}
        for item in document["dependencies"]:
            self.assertIn(item["ref"], refs)
            self.assertTrue(set(item["dependsOn"]) <= refs)


if __name__ == "__main__":
    unittest.main()
