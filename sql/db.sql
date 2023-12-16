SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

-- ----------------------------
-- Table structure for changes
-- ----------------------------
DROP TABLE IF EXISTS `changes`;
CREATE TABLE `changes`  (
  `code` mediumint NOT NULL,
  `start` smallint NOT NULL,
  `new_code` mediumint NOT NULL,
  `time` smallint NOT NULL,
  `details_id` int NULL DEFAULT NULL,
  PRIMARY KEY (`code`, `start`, `new_code`, `time`) USING BTREE,
  INDEX `details_id`(`details_id` ASC) USING BTREE,
  CONSTRAINT `changes_ibfk_1` FOREIGN KEY (`code`, `start`) REFERENCES `codes` (`code`, `start`) ON DELETE RESTRICT ON UPDATE RESTRICT,
  CONSTRAINT `changes_ibfk_2` FOREIGN KEY (`details_id`) REFERENCES `details` (`id`) ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE = InnoDB CHARACTER SET = utf8mb4 COLLATE = utf8mb4_zh_0900_as_cs;

-- ----------------------------
-- Table structure for codes
-- ----------------------------
DROP TABLE IF EXISTS `codes`;
CREATE TABLE `codes`  (
  `code` mediumint NOT NULL,
  `name` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_zh_0900_as_cs NOT NULL,
  `start` smallint NOT NULL,
  `end` smallint NULL DEFAULT NULL,
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

SET FOREIGN_KEY_CHECKS = 1;
