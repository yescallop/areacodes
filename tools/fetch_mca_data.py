import json
import os
import argparse
import urllib.request
from pathlib import Path

def pad_code(code):
    return code.ljust(6, '0')

def collect_codes(node, results):
    code = node.get("code")
    name = node.get("name")
    
    # Skip root node or nodes without code/name if any
    if code == "00":
        pass
    # Handle Taiwan specially as per original logic
    elif code == "资料暂缺" and name == "台湾省":
        code = "71"
        results.append((pad_code(code), name))
    elif code and name:
        results.append((pad_code(code), name))
        
    children = node.get("children")
    if children:
        for child in children:
            collect_codes(child, results)

def main():
    parser = argparse.ArgumentParser(description="Fetch MCA data and generate year file.")
    parser.add_argument("year", help="The year for the output file name (e.g. 2025)")
    args = parser.parse_args()

    url = "https://dmfw.mca.gov.cn/xzqh/getList?code=0&trimCode=true&maxLevel=3"
    
    # Calculate output path relative to this script: ../data/{year}.txt
    # Assuming script is in tools/ and data is in data/ sibling to tools/
    script_dir = Path(__file__).resolve().parent
    output_path = script_dir.parent / "data" / f"{args.year}.txt"
    
    print(f"Fetching data from {url}...")
    try:
        # Using a browser-like User-Agent to avoid potential blocking
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        }
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
    except Exception as e:
        print(f"Error fetching data: {e}")
        return

    print("Processing data...")
    root = data.get('data')
    if not root:
        print("Error: 'data' field not found in JSON response")
        return

    results = []
    collect_codes(root, results)
    
    # Sort by code
    results.sort(key=lambda x: x[0])
    
    # Generate content
    lines = [f"{code} {name}" for code, name in results]
    new_content = "\n".join(lines) + "\n"
    
    print(f"Writing to {output_path}...")
    
    # Ensure directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
        
    print("Done.")

if __name__ == "__main__":
    main()
