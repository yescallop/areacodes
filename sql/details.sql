BEGIN;
INSERT INTO `details` (`text`) VALUES
('黑民行批〔2022〕7号：七台河市新兴区设立兴北镇，隶属关系由新兴区划归茄子河区管辖。'),
('赣府字〔2022〕36号：将南昌市新建区流湖镇划归南昌市红谷滩区管辖。'),
('豫民行批〔2022〕1号：将尉氏县岗李乡、大马乡行政区划转新郑市，经济社会管理职能委托郑州航空港区管理。'),
('甘政发〔2022〕14号：将皋兰县忠和镇的罗官村、忠和村、水源村3个建制村划入城关区，其中罗官村、忠和村2个建制村划入城关区盐场路街道管辖，水源村1个建制村划入城关区青白石街道管辖。将皋兰县九合镇及其所辖全部11个建制村，忠和镇及其所辖的盐池社区1个社区和崖川村、丰登村、平岘村、六合村、盐池村5个建制村划入安宁区。'),
('宁政函〔2022〕31号：将贺兰县北绕城高速路以南，第二农场渠向南延伸至唐徕渠与金凤区—贺兰县现行政区域界线交汇处以西，金凤区与贺兰县现行政区域界线以东区域调整变更至金凤区管辖。');
SET @id = LAST_INSERT_ID();

UPDATE `changes` SET `details_id` = @id WHERE (`code`, `new_code`, `time`) = (230902, 230904, 2022);
UPDATE `changes` SET `details_id` = @id := @id + 1 WHERE (`code`, `new_code`, `time`) = (360112, 360113, 2022);
UPDATE `changes` SET `details_id` = @id := @id + 1 WHERE (`code`, `new_code`, `time`) = (410223, 410184, 2022);
UPDATE `changes` SET `details_id` = @id := @id + 1 WHERE (`code`, `new_code`, `time`) = (620122, 620102, 2022);
UPDATE `changes` SET `details_id` = @id WHERE (`code`, `new_code`, `time`) = (620122, 620105, 2022);
UPDATE `changes` SET `details_id` = @id := @id + 1 WHERE (`code`, `new_code`, `time`) = (640122, 640106, 2022);
COMMIT;
