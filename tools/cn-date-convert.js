#!/usr/bin/env node
/**
 * cn-date-convert.js
 *
 * 用法:
 *   node cn-date-convert.js input.txt output.txt
 *   node cn-date-convert.js file.txt         # 覆盖写回同一文件
 *
 * 功能:
 *   将文本中的中文日期形式 "一九八六年五月二十七日" => "1986年5月27日"
 *   支持的中文数字: 零 〇 一 二 三 四 五 六 七 八 九 两 十 廿 卅
 */

const fs = require('fs');
const path = require('path');

if (process.argv.length < 3) {
  console.error('Usage: node cn-date-convert.js <input-file> [output-file]');
  process.exit(1);
}

const inputPath = process.argv[2];
const outputPath = process.argv[3] || inputPath;

// 映射：中文数字字符 -> 数字（用于年份按位转换和单个数字）
const DIGIT_MAP = {
  '零': 0, '〇': 0,
  '一': 1, '二': 2, '两': 2, '三': 3, '四': 4, '五': 5,
  '六': 6, '七': 7, '八': 8, '九': 9,
};

// 解析年份：按字符逐位转换（例如 "一九八六" -> "1986"）
function parseYear(chineseYear) {
  let digits = [];
  for (let ch of chineseYear) {
    if (ch in DIGIT_MAP) {
      digits.push(DIGIT_MAP[ch].toString());
    } else if (ch === '十') {
      // 若年份里出现“十”，把它当作 '10' 的一位 '10' 不合理，退回为0占位（极少见）
      digits.push('10');
    } else {
      // 非预期字符，忽略或保留原样（这里选择忽略）
    }
  }
  return digits.join('');
}

// 解析月/日：支持 "五", "十一", "二十七", "十", "廿五", "卅一" 等
function parseMonthOrDay(chineseNum) {
  if (!chineseNum || chineseNum.length === 0) return NaN;

  // 直接单字符映射
  if (chineseNum.length === 1 && chineseNum in DIGIT_MAP) {
    return DIGIT_MAP[chineseNum];
  }

  // 特殊字符
  if (chineseNum === '十') return 10;
  if (chineseNum === '廿') return 20;
  if (chineseNum === '卅') return 30;

  // 处理包含“十”的情况
  if (chineseNum.includes('十')) {
    // 形式：十、十二、二十、二十三、十三
    const parts = chineseNum.split('十');
    const left = parts[0]; // 可能为空 (表示 1*10)
    const right = parts[1]; // 可能为空
    let leftVal = 0;
    let rightVal = 0;

    if (left === '') leftVal = 1;
    else if (left in DIGIT_MAP) leftVal = DIGIT_MAP[left];
    else leftVal = 0;

    if (right === '') rightVal = 0;
    else if (right in DIGIT_MAP) rightVal = DIGIT_MAP[right];
    else {
      // right 可能是多字符（不常见），逐字拼接
      let rv = '';
      for (let ch of right) {
        if (ch in DIGIT_MAP) rv += DIGIT_MAP[ch];
      }
      rightVal = rv ? parseInt(rv, 10) : 0;
    }

    return leftVal * 10 + rightVal;
  }

  // 处理包含“廿”或“卅”的情况，比如 "廿五"、"卅一"
  if (chineseNum.startsWith('廿')) {
    const tail = chineseNum.slice(1);
    const tailVal = tail ? (tail in DIGIT_MAP ? DIGIT_MAP[tail] : 0) : 0;
    return 20 + tailVal;
  }
  if (chineseNum.startsWith('卅')) {
    const tail = chineseNum.slice(1);
    const tailVal = tail ? (tail in DIGIT_MAP ? DIGIT_MAP[tail] : 0) : 0;
    return 30 + tailVal;
  }

  // 退化：逐字拼接（例如 "二七" -> 27 的情形非常罕见，但仍做尝试）
  let s = '';
  for (let ch of chineseNum) {
    if (ch in DIGIT_MAP) s += DIGIT_MAP[ch];
  }
  if (s) return parseInt(s, 10);

  return NaN;
}

// 主替换逻辑：匹配 年/月/日 三部分
// 说明：正则尽量宽松匹配常见中文数字与字符（含廿卅两十等）
const dateRegex = /([零〇一二三四五六七八九两十百千]+?)年([零〇一二三四五六七八九两十廿卅]+?)月([零〇一二三四五六七八九两十廿卅]+?)日/g;

function convertText(text) {
  return text.replace(dateRegex, (match, yearPart, monthPart, dayPart) => {
    // 清理空格
    yearPart = yearPart.replace(/\s+/g, '');
    monthPart = monthPart.replace(/\s+/g, '');
    dayPart = dayPart.replace(/\s+/g, '');

    const yearNumStr = parseYear(yearPart);
    const monthNum = parseMonthOrDay(monthPart);
    const dayNum = parseMonthOrDay(dayPart);

    // 若解析失败，则返回原文（不破坏）
    if (!yearNumStr || isNaN(monthNum) || isNaN(dayNum)) {
      return match;
    }

    return `${yearNumStr}年${parseInt(monthNum, 10)}月${parseInt(dayNum, 10)}日`;
  });
}

// 读取、转换、写回
try {
  const raw = fs.readFileSync(inputPath, 'utf8');
  const converted = convertText(raw);
  fs.writeFileSync(outputPath, converted, 'utf8');
  console.log(`转换完成：${inputPath} -> ${outputPath}`);
} catch (err) {
  console.error('处理文件时出错：', err.message);
  process.exit(2);
}
