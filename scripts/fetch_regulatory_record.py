"""Fetch and preserve a single fixed-origin regulatory source for SearchRight's CLI."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile
from urllib.parse import urlencode

EMA = "https://www.ema.europa.eu/en/documents/report/medicines-output-medicines_json-report_en.json"
FDA = "https://api.fda.gov/drug/drugsfda.json"
LIMIT = 64 * 1024 * 1024

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("provider", choices=("drugsfda", "ema-medicines"))
    parser.add_argument("identifier")
    parser.add_argument("destination", type=Path)
    parser.add_argument("--searchright", default="searchright")
    args = parser.parse_args()
    pattern = r"(?:BLA|NDA|ANDA)[0-9]{6}" if args.provider == "drugsfda" else r"EMEA/H/[A-Z]/[0-9]+"
    if not re.fullmatch(pattern, args.identifier):
        parser.error("unexpected publisher identifier")
    endpoint = EMA if args.provider == "ema-medicines" else FDA + "?" + urlencode({
        "search": f'application_number:"{args.identifier}"', "limit": 1})
    dest = args.destination.resolve()
    if dest.exists():
        parser.error("destination already exists")
    with tempfile.TemporaryDirectory(dir=dest.parent) as temp:
        packet = Path(temp)/"packet"
        packet.mkdir()
        raw = packet/"source.json"
        result = subprocess.run(["curl","--disable","--silent","--show-error","--fail",
            "--proto","=https","--max-time","90","--max-filesize",str(LIMIT),
            "--output",str(raw),"--write-out","%{http_code}",endpoint],check=True,capture_output=True,text=True)
        if result.stdout != "200":
            raise ValueError(f"publisher returned {result.stdout}")
        record = subprocess.check_output([args.searchright,"source","regulatory-record",args.provider,args.identifier,str(raw)])
        (packet/"record.json").write_bytes(record)
        receipt = dict(provider=args.provider,identifier=args.identifier,url=endpoint,
            retrieved_at=datetime.datetime.now(datetime.UTC).isoformat(),
            source_sha256=hashlib.file_digest(raw.open("rb"),"sha256").hexdigest(),
            record_sha256=hashlib.sha256(record).hexdigest())
        (packet/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
        packet.rename(dest)

if __name__ == "__main__":
    main()
