# The diff specification used in this project (outdated)

## Overview

This diff specification, called `area-diff`, is based on `git-diff` with some modifications. It is designed to accurately describe the correspondence between old and new division codes, under the following four principles.

- Rigorous: the correspondence of old and new division codes are defined in terms of the intersection of administrative areas.
- Concise: the separators are all single characters, as few as possible while ensuring clarity.
- Maintainable: the syntax is intuitive and uses division names to describe the correspondence between codes, which facilitates data entry and proofreading.
- Analyzable: a program with simple algorithms can analyze the processed data for the correspondence between codes and present it clearly.

## Background

1. Each file under the `data` directory is called a **data table** of division codes. Each row in a data table is called a **record** of division code. A record consists of its **code** and its **name**, separated by a tab `\t`. Each record corresponds to a certain range of administrative area, called the **area of the record**.
1. Each file under the `diff` directory is called a **diff table**, which contains the difference of records in the two corresponding data tables. Among them, the table used as a relative reference is called the **source table**, while the other is called the **destination table**. The **original contents** of a diff table are obtained by calling `git diff -U0 --no-index` on the corresponding data tables and removing the irrelevant or duplicate rows. The rows with contents identical to the original ones in a diff table are called **original rows**.

## Detailed rules

1. Each non-empty row in a diff table is divided into four types, determined by its initial character: **deletion row** (`-`), **addition row** (`+`), **internal change row** (`=`) and **comment row** (`#`). In particular, internal change rows and non-original deletion and addition rows are collectively referred to as **change rows**.
1. Any modification in a diff table should obey the following rules.
    - In a diff table, it is not permitted to delete any row that contains the original contents, to modify the original contents therein, or to insert any deletion or addition row, unless the corresponding data tables have been revised.
    - If a row in a diff table contains the original contents, the original contents therein should always be kept at the start of the row and only the contents after the original ones to the end of the row may be modified.
    - All rows in a diff table are not sequential and can thus be sorted in any order, but care should be taken to maintain a certain structure.
    - The syntax of all rows in a diff table must strictly adhere to the rules hereafter.
1. Each change row has its corresponding record in the corresponding data tables, called the **record of the row**. The record of a deletion row is in the source table, that of an addition row in the destination table, and that of an internal change row in both the source and destination tables.
1. A change row consists of its initial character, its record and its **attribute**, joined in order. The attribute of a change row has a defined syntax and describes how the areas of the row's record and of the associated records intersect. It is hereafter assumed that a row is a change row when referring to its attribute.
1. The attribute of a deletion (an addition) row specifies in the destination (source) table, any sibling record such that its area intersects with the area of the row's record. Thereafter, if the areas of the specified sibling records still cannot cover the full area of the row's record, then the attribute of the row should additionally specify all records in the parent level that satisfy the aforementioned requirements. Comments should be included when a record in the parent level is specified, and implementations should ask the user to confirm in such a case.
1. The attribute of a deletion row starts with the character `>`, followed by one or more **record selectors** separated by the character `,`. The attribute of an addition row starts with the character `<`, the rest of which shares the same syntax as a deletion row. The attribute of an internal change row has the same syntax either as a deletion row or as an addition row.
1. By altering the initial character of an internal change row, one can obtain the corresponding deletion or addition row. The records specified by the attribute of an internal change row are the result of removing the row's record from the records specified by the attribute of its corresponding deletion or addition row.
1. There are five types of record selectors: **specified name**, **current name**, **current code**, **parent code** and **doubt**. Among them, the first four selectors are collectively referred to as **general** selectors. They are defined as follows.
    - A **specified name** selector is text-valued, selecting the record with the same name as its value and the least distance from the record of the current row.
    - A **current name** selector has a value of `#`, selecting the record with the same name as and the least distance from the record of the current row.
    - A **current code** selector has a value of `.`, selecting the record with the same code as the record of the current row.
    - A **parent code** selector has a value of `..`, selecting the record corresponding to the parent code of the code of the record of the current row.
    - A **doubt** selector consists of the value of a general selector followed by a **doubt flag**. The available values for a doubt flag are `?` and `!`, where `?` indicates that the selector is invalid and disabled, generally used when the associated records cannot be found; while `!` indicates that the selector is valid and selecting the same records as the embedded general selector, generally used when the associated records can be found without an official interpretation but there is a good reason to enable the selector.
1. The possible values of the distance between two records are:
    - 0 (manually selected by the user).
    - 1 (in the same second-level division).
    - 2 (in the same first-level division but different second-level divisions).
    - 3 (in different first-level divisions).

    Implementations that adhere to this specification, when selecting the record with the least distance from the specified record, should first automatically determine whether such a record is unique, and if it is unique, select that unique record; if it is not unique, leave it to the user to select.

## Note

1. When entering textual descriptions of division changes into a diff table, additional consideration should be given to the presence of internal changes, which can be obtained by searching the page for the keyword "划归" ("transfer ... to the jurisdiction of ...") and filtering manually. After each year's internal changes are entered, the internal changes should be documented by year in [diff-note.md](diff-note.md). If there are none, document "无" ("none").

## Examples

- Abolish Chongchuan District (崇川区) and Gangzha District (港闸区). Create a new Chongchuan District in Nantong City (南通市), the administrative area of which covers those of the former Chongchuan District and Gangzha District.

    ```diff
    -320602	崇川区>#
    -320611	港闸区>崇川区
    +320613	崇川区<#,港闸区
    ```

- Rename Qiaoxi District (桥西区) in Xingtai City (邢台市) as Xindu District (信都区).

    ```diff
    -130503	桥西区>.
    +130503	信都区<.
    ```

- Transfer Gaoyu Town, Guanglu Town, Xinzhuangying Township, Huaguanying Township and Taicheng Township in Ci County (磁县) to the jurisdiction of Hanshan District (邯山区), Handan City (邯郸市). Transfer Lintan Town and Nancheng Township in Ci County to the jurisdiction of Fuxing District (复兴区), Handan City.

    Transfer part of the areas of Hejing County (和静县), Yanqi Hui Autonomous County (焉耆回族自治县), Bohu County (博湖县), Heshuo County (和硕县), Ruoqiang County (若羌县) and Qiemo County (且末县) in Bayingolin Mongol Autonomous Prefecture (巴音郭楞蒙古自治州) to the jurisdiction of Tiemenguan City (铁门关市).

    ```diff
    =130427	磁县>邯山区,复兴区
    =659006	铁门关市<和静县,焉耆回族自治县,博湖县,和硕县,若羌县,且末县
    ```

- For more examples: see the completed diff tables (especially of 2004-2005) and the corresponding textual descriptions.
