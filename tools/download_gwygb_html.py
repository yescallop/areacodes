import requests

from bs4 import BeautifulSoup
from requests.compat import urljoin
from time import sleep
from utils import download

json_url = 'http://www.gov.cn/gbgl/xhtml/js/gbgl.json'


def main():
    session = requests.Session()
    r = session.get(json_url)
    years = r.json()[0]['values']

    for year, issues in years.items():
        year = year.removeprefix('y')

        for issue in issues.values():
            issue_no = issue['issue']
            issue_url = urljoin(json_url, issue['gname'])

            print(f"{year}年第{issue_no}号（总号：{issue['serial']}）")
            download_issue(session, year, issue_no, issue_url)
            sleep(1)


def download_issue(session, year, issue_no, issue_url):
    r = session.get(issue_url, stream=True)
    soup = BeautifulSoup(r.raw, 'html.parser')
    article_no = 1

    for link in soup.select('ul a'):
        try:
            ul_class = link.parent.parent['class']
        except:
            continue
        if 'table_contents_list' in ul_class or 'list01' in ul_class:
            issue_id = f'{year}{issue_no:02}'
            article_id = f'{issue_id}{article_no:02}'
            url = urljoin(json_url, link.get('href'))
            if download(url, f"gwygb/{year}/{issue_id}/{article_id}.html", session):
                sleep(1)
            article_no += 1


main()
