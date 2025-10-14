set search_path to process_log

CREATE TYPE process_log.STATE AS ENUM(
	'Processing',
	'ProcessDone',
	'Abnormal'
);

DROP TABLE process_log.CR1;
CREATE TABLE process_log.CR1(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	task_id INT,
	state process_log.STATE NOT NULL,
	data_start TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	data_end TIMESTAMP,
	process_duration TIME,
	recipe_name TEXT NOT NULL,
	count INT
);

SELECT * FROM process_log.CR1;
