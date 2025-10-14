CREATE TABLE cache (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated TEXT NOT NULL
);

CREATE TABLE anmeldungen_plan (
    course_of_study TEXT NOT NULL, -- TODO FIXME the url contains this at the start so this is duplicate
    url TEXT NOT NULL,
    name TEXT NOT NULL,
    parent TEXT REFERENCES anmeldungen (url),
    min_cp INT NOT NULL,
    max_cp INT,
    min_modules INT NOT NULL,
    max_modules INT,
    PRIMARY KEY (course_of_study, url)
) STRICT;

CREATE TABLE anmeldungen_entries (
    course_of_study TEXT NOT NULL,
    available_semester TEXT NOT NULL, -- s or w or b
    anmeldung TEXT,
    module_url TEXT, -- thesis, anerkannt etc can be null
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    credits INT NOT NULL,
    state TEXT NOT NULL, -- not_planned or planned or done
    year INT,
    semester TEXT,
    -- TODO only make id the primary key
    PRIMARY KEY (course_of_study, anmeldung, available_semester, id),
    FOREIGN KEY (course_of_study, anmeldung) REFERENCES anmeldungen (course_of_study, url)
) STRICT;
