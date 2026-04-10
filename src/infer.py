import sys, os, argparse, json
import google.genai as genai
import google.genai.types as t

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--images', required=True)
    p.add_argument('--prompt', required=True)
    args = p.parse_args()

    key = os.environ.get('GEMINI_API_KEY')
    if not key:
        print('GEMINI_API_KEY not set', file=sys.stderr)
        sys.exit(1)

    client = genai.Client(api_key=key)
    paths = [x.strip() for x in args.images.split(',') if x.strip()]
    parts = []
    for path in paths:
        with open(path, 'rb') as f:
            parts.append(t.Part.from_bytes(data=f.read(), mime_type='image/png'))
    parts.append(t.Part.from_text(text=args.prompt))

    resp = client.models.generate_content(
        model='gemini-2.0-flash',
        contents=t.Content(role='user', parts=parts),
        config=t.GenerateContentConfig(max_output_tokens=4096)
    )
    raw = resp.text
    s = raw.strip()
    for prefix in ('```json', '```'):
        if s.startswith(prefix):
            s = s[len(prefix):]
    s = s.rstrip('`').strip()
    print(s)

main()
