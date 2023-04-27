import re
import requests

from bs4 import BeautifulSoup
from requests.compat import urljoin
from time import sleep
from utils import download

main_url = 'http://www.gov.cn/zhengce/gongbao/guowuyuan1954-1999.htm'
pat = re.compile(r'中华人民共和国国务院公报（(\d{4})年）')


def main():
    session = requests.Session()
    r = session.get(main_url, stream=True)
    soup = BeautifulSoup(r.raw, 'html.parser')

    for link in soup.find_all('a'):
        m = pat.match(link.string)
        if m:
            download_year(session, m.group(1), urljoin(main_url, link.get('href')))
            sleep(1)


def download_year(session, year, year_url):
    r = session.get(year_url, stream=True)
    soup = BeautifulSoup(r.raw, 'html.parser')

    for link in soup.find_all('a'):
        url = link.get('href')
        if url and url.endswith('.pdf'):
            title = link.strings.__next__()
            filename = url.split('/')[-1].removeprefix('gwyb')
            
            print(title)
            if download(url, f"gwygb/{year}/{filename}", session):
                sleep(1)


main()
