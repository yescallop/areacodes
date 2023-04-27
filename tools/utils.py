import functools
import pathlib
import requests
import shutil

from tqdm.auto import tqdm


def download(url: str, filename: str, session=None) -> bool:
    path = pathlib.Path(filename).expanduser().resolve()
    if path.exists():
        return False
    path.parent.mkdir(parents=True, exist_ok=True)

    tmp_path = path.with_suffix(path.suffix + '.tmp')

    r = session.get(url, stream=True, allow_redirects=True) if session else requests.get(
        url, stream=True, allow_redirects=True)
    if r.status_code != 200:
        r.raise_for_status()  # Will only raise for 4xx codes, so...
        raise RuntimeError(
            f"Request to {url} returned status code {r.status_code}")
    file_size = int(r.headers.get('Content-Length', 0))

    r.raw.read = functools.partial(
        r.raw.read, decode_content=True)  # Decompress if needed
    with tqdm.wrapattr(r.raw, "read", total=file_size, desc=path.name) as r_raw:
        with tmp_path.open("wb") as f:
            shutil.copyfileobj(r_raw, f)

    shutil.move(tmp_path, path)
    return True
