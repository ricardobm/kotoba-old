import * as requestModule from 'request'
import * as util from 'util'

const JISHO_API_URI = 'https://jisho.org/api/v1/search/words'
const JISHO_API_TIMEOUT_MS = 2500

export type JishoArgs = {
    term: string,
    withSound: boolean,
}

const request = util.promisify(requestModule)

export async function queryJisho(args: JishoArgs) {
    const params = { keyword: args.term }
    const response = await request({
        uri:     JISHO_API_URI,
        qs:      params,
        method:  'GET',
        json:    true,
        timeout: JISHO_API_TIMEOUT_MS,
    })
    const body = response.body as jishoResponse
    if (!body || !body.meta || body.meta.status !== 200) {
        throw new Error('Invalid Jisho response')
    }

    // Sanitize the response
    const data = body.data || []
    for (const it of data) {
        for (const japanese of it.japanese) {
            // Word can be undefined for kana-only terms.
            japanese.word = japanese.word || japanese.reading
        }
    }

    return data
}

/** Root response from Jisho. */
type jishoResponse = {
    data: JishoEntry[],
    meta: {
        status: number,
    },
}

/** Entry from the Jisho response. */
export type JishoEntry = {
    /**
     * Slug is the Japanese word (e.g. `家`) possibly with an additional
     * counter (e.g. `家-1`).
     */
    slug: string,

    /** Is this a common definition? */
    is_common: boolean,

    /**
     * Japanese terms for this entry. The first one is the main term (e.g. the
     * most common) while others are additional forms.
     */
    japanese: JishoJapanese[],

    /**
     * The actual translations (i.e. senses) for the entry.
     */
    senses: JishoEnglish[],

    /** JLPT tags (e.g. `jlpt-n5`). */
    jlpt: string[],

    //
    // The entries below are not that useful, but are here for completeness:
    //

    /** Entry tags (e.g. `wanikani8`). */
    tags: string[],

    /** Attribution sources for this definition. */
    attribution: {[key: string]: boolean|string},
}

/**
 * A Japanese term for a `JishoEntry`. A single entry can have multiple terms.
 */
export type JishoJapanese = {

    /**
     * The japanese word for this term.
     */
    word: string,

    /** The reading for this term in kana. */
    reading: string,
}

/**
 * One English sense for a `JishoEntry`. A single entry can have multiple
 * senses.
 */
export type JishoEnglish = {

    /** List of english definitions for the entry. */
    english_definitions: string[],

    /**
     * Additional human readable tags for the entry.
     *
     * Examples: `Usually written using kana alone`, `Abbreviation`
     */
    tags: string[],

    /**
     * List of parts of speech for the entry. Those are human readable.
     *
     * Examples: `Noun`, `Place`, `Na-adjective`, `Expression`, `Suru verb`
     * `Adverb taking the 'to' particle`.
     */
    parts_of_speech: string[],

    /** Related dictionary entries. */
    see_also: string[],

    /** Extra information about the entry (e.g. `from 〜のうち`). */
    info: string[],

    /** Related links (e.g. from Wikipedia). */
    links: Array<{text: string, url: string}>,

    // Couldn't find a sample for the fields below:

    /** Unused */
    antonyms: any[],

    /** Unused */
    source: any[],

    /** Unused */
    restrictions: any[],
}
