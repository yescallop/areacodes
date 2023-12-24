SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

-- ----------------------------
-- Table structure for changes
-- ----------------------------
DROP TABLE IF EXISTS `changes`;
CREATE TABLE `changes`  (
  `code` int NOT NULL,
  `new_code` int NOT NULL,
  `time` int NOT NULL,
  `details_id` int NULL DEFAULT NULL,
  PRIMARY KEY (`code`, `new_code`, `time`) USING BTREE,
  INDEX `details_id`(`details_id` ASC) USING BTREE,
  CONSTRAINT `changes_ibfk_1` FOREIGN KEY (`details_id`) REFERENCES `details` (`id`) ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs;

-- ----------------------------
-- Table structure for codes
-- ----------------------------
DROP TABLE IF EXISTS `codes`;
CREATE TABLE `codes`  (
  `code` int NOT NULL,
  `name` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_zh_0900_as_cs NOT NULL,
  `start` int NOT NULL,
  `end` int NULL DEFAULT NULL,
  PRIMARY KEY (`code`, `start`) USING BTREE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs;

-- ----------------------------
-- Table structure for details
-- ----------------------------
DROP TABLE IF EXISTS `details`;
CREATE TABLE `details`  (
  `id` int NOT NULL AUTO_INCREMENT,
  `text` text CHARACTER SET utf8mb4 COLLATE utf8mb4_zh_0900_as_cs NOT NULL,
  PRIMARY KEY (`id`) USING BTREE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs;

-- ----------------------------
-- View structure for changes_ext
-- ----------------------------
DROP VIEW IF EXISTS `changes_ext`;
CREATE ALGORITHM = MERGE SQL SECURITY DEFINER VIEW `changes_ext` AS select `c`.`code` AS `code`,`p`.`name` AS `name`,`p`.`start` AS `start`,`c`.`new_code` AS `new_code`,`s`.`name` AS `new_name`,`s`.`start` AS `new_start`,`c`.`time` AS `time`,`c`.`details_id` AS `details_id` from ((`changes` `c` join `codes` `p` on(((`p`.`code` = `c`.`code`) and ((`c`.`time` - 1) >= `p`.`start`) and ((`p`.`end` is null) or ((`c`.`time` - 1) < `p`.`end`))))) join `codes` `s` on(((`s`.`code` = `c`.`new_code`) and (`c`.`time` >= `s`.`start`) and ((`s`.`end` is null) or (`c`.`time` < `s`.`end`)))));

SET FOREIGN_KEY_CHECKS = 1;
