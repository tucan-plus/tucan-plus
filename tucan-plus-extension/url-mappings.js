/**
 * @typedef MappingType
 * @type {object}
 * @property {string} from
 * @property {string|null} to null means interpolate here
 */

/**
 * 
 * @param {TemplateStringsArray} strings 
 * @param {MappingType[]} args
 * @returns {{ strings: TemplateStringsArray; args: MappingType[]; }} 
 */
function t(strings, ...args) {
    return {
        strings,
        args
    }
}

/**
 * 
 * @param {string|number|undefined} number
 * @returns {MappingType} 
 */
function num(number) {
    if (number === undefined) {
        return {
            from: "DONTMATCH",
            to: "DONTMATCH"
        }
    } else {
        return {
            from: "\\d+",
            to: number.toString()
        }
    }
}

/**
 * 
 * @param {string} string
 * @returns {MappingType} 
 */
function s(string) {
    return {
        from: string,
        to: null // null means interpolate here
    }
}

export const bidirectionalMappings = (/** @type {string|number|undefined} */ id) => [
    {
        "tucan": t`PRGNAME=MODULEDETAILS&ARGUMENTS=-N${num(id)},-N${num(275)},${s("(.*)")}`,
        "tucan-plus": t`module-details/${s("(.*)")}`,
    },
    {
        "tucan": t`PRGNAME=COURSEDETAILS&ARGUMENTS=-N${num(id)},-N${num(274)},${s("(.*)")}`,
        "tucan-plus": t`course-details/${s("(.*)")}`,
    },
    {
        "tucan": t`PRGNAME=REGISTRATION&ARGUMENTS=-N${num(id)},-N${num(311)},${s("(.*)")}`,
        "tucan-plus": t`registration/${s("(.*)")}`,
    },
    {
        "tucan": t`PRGNAME=MLSSTART&ARGUMENTS=-N${num(id)},-N${num(19)},`,
        "tucan-plus": t`overview`,
    },
    {
        "tucan": t`PRGNAME=ACTION&ARGUMENTS=${s("(.*)")}`,
        "tucan-plus": t`vv/${s("(.*)")}`,
    },
    {
        "tucan": t`PRGNAME=MYMODULES&ARGUMENTS=-N${num(id)},-N${num(275)},`,
        // TODO FIXME my-modules/current this is actually not current but recent as it remembers last navigation
        "tucan-plus": t`my-modules/current`,
    },
    {
        "tucan": t`PRGNAME=MYMODULES&ARGUMENTS=-N${num(id)},-N${num(275)},-N999`,
        "tucan-plus": t`my-modules/all`,
    },
    {
        "tucan": t`PRGNAME=MYMODULES&ARGUMENTS=-N${num(id)},-N${num(275)},-N${s("(\\d+)")}`,
        "tucan-plus": t`my-modules/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=PROFCOURSES&ARGUMENTS=-N${num(id)},-N${num(274)},`,
        "tucan-plus": t`my-courses/current`,
    },
    {
        "tucan": t`PRGNAME=PROFCOURSES&ARGUMENTS=-N${num(id)},-N${num(274)},-N999`,
        "tucan-plus": t`my-courses/all`,
    },
    {
        "tucan": t`PRGNAME=PROFCOURSES&ARGUMENTS=-N${num(id)},-N${num(274)},-N${s("(\\d+)")}`,
        "tucan-plus": t`my-courses/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=MYEXAMS&ARGUMENTS=-N${num(id)},-N${num(318)},`,
        "tucan-plus": t`my-exams/current`,
    },
    {
        "tucan": t`PRGNAME=MYEXAMS&ARGUMENTS=-N${num(id)},-N${num(318)},-N999`,
        "tucan-plus": t`my-exams/all`,
    },
    {
        "tucan": t`PRGNAME=MYEXAMS&ARGUMENTS=-N${num(id)},-N${num(318)},-N${s("(\\d+)")}`,
        "tucan-plus": t`my-exams/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=EXAMREGISTRATION&ARGUMENTS=-N${num(id)},-N${num(318)},`,
        "tucan-plus": t`exam-registration/current`,
    },
    {
        "tucan": t`PRGNAME=EXAMREGISTRATION&ARGUMENTS=-N${num(id)},-N${num(318)},-N${s("(\\d+)")}`,
        "tucan-plus": t`exam-registration/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=EXAMRESULTS&ARGUMENTS=-N${num(id)},-N${num(325)},`,
        "tucan-plus": t`exam-results/current`,
    },
    {
        "tucan": t`PRGNAME=EXAMRESULTS&ARGUMENTS=-N${num(id)},-N${num(325)},-N999`,
        "tucan-plus": t`exam-results/all`,
    },
    {
        "tucan": t`PRGNAME=EXAMRESULTS&ARGUMENTS=-N${num(id)},-N${num(325)},-N${s("(\\d+)")}`,
        "tucan-plus": t`exam-results/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=COURSERESULTS&ARGUMENTS=-N${num(id)},-N${num(324)},`,
        "tucan-plus": t`course-results/current`,
    },
    {
        "tucan": t`PRGNAME=COURSERESULTS&ARGUMENTS=-N${num(id)},-N${num(324)},-N${s("(\\d+)")}`,
        "tucan-plus": t`course-results/${s("(\\d+)")}`,
    },
    {
        "tucan": t`PRGNAME=CREATEDOCUMENT&ARGUMENTS=-N${num(id)},-N${num(557)},`,
        "tucan-plus": t`my-documents`,
    },
    {
        "tucan": t`PRGNAME=EXTERNALPAGES&ARGUMENTS=-N000000000000001,-N000344,-Awelcome`,
        "tucan-plus": t``,
    },
    {
        "tucan": t`PRGNAME=STUDENT_RESULT&ARGUMENTS=-N${num(id)},-N${num(316)},-N0,-N000000000000000,-N000000000000000,-N000000000000000,-N0,-N000000000000000`,
        "tucan-plus": t`student-result/default`,
    },
    {
        "tucan": t`PRGNAME=STUDENT_RESULT&ARGUMENTS=-N${num(id)},-N${num(316)},-N0,-N000000000000000,-N000000000000000,-N${s("(\\d+)")},-N0,-N000000000000000`,
        "tucan-plus": t`student-result/${s("(\\d+)")}`,
    },
]