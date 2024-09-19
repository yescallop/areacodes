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
  `desc_id` int NULL DEFAULT NULL,
  PRIMARY KEY (`code`, `new_code`, `time`) USING BTREE,
  INDEX `desc_id`(`desc_id` ASC) USING BTREE,
  CONSTRAINT `changes_ibfk_1` FOREIGN KEY (`desc_id`) REFERENCES `descriptions` (`id`) ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs ROW_FORMAT = DYNAMIC;

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
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs ROW_FORMAT = DYNAMIC;

-- ----------------------------
-- Table structure for descriptions
-- ----------------------------
DROP TABLE IF EXISTS `descriptions`;
CREATE TABLE `descriptions`  (
  `id` int NOT NULL AUTO_INCREMENT,
  `text` text CHARACTER SET utf8mb4 COLLATE utf8mb4_zh_0900_as_cs NOT NULL,
  PRIMARY KEY (`id`) USING BTREE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs ROW_FORMAT = DYNAMIC;

-- ----------------------------
-- View structure for changes_ext
-- ----------------------------
DROP VIEW IF EXISTS `changes_ext`;
CREATE ALGORITHM = MERGE SQL SECURITY DEFINER VIEW `changes_ext` AS select `c`.`code` AS `code`,`a`.`name` AS `name`,`a`.`start` AS `start`,`c`.`new_code` AS `new_code`,`b`.`name` AS `new_name`,`b`.`start` AS `new_start`,`c`.`time` AS `time`,`c`.`desc_id` AS `desc_id`,`d`.`text` AS `desc_text` from (((`changes` `c` join `codes` `a` on(((`a`.`code` = `c`.`code`) and (`a`.`start` < `c`.`time`) and ((`a`.`end` is null) or (`a`.`end` >= `c`.`time`))))) join `codes` `b` on(((`b`.`code` = `c`.`new_code`) and (`b`.`start` <= `c`.`time`) and ((`b`.`end` is null) or (`b`.`end` > `c`.`time`))))) left join `descriptions` `d` on((`d`.`id` = `c`.`desc_id`))) order by `c`.`time` desc,`c`.`code`,`c`.`new_code`;

SET FOREIGN_KEY_CHECKS = 1;
