import * as requestModule from 'request'
import * as util from 'util'
import { JSDOM } from 'jsdom'

const JISHO_API_URI = 'https://jisho.org/api/v1/search/words'
const JISHO_API_TIMEOUT_MS = 2500

const JISHO_SEARCH_URI = 'https://jisho.org/search/'
const JISHO_SEARCH_TIMEOUT_MS = 3500

export type JishoArgs = {
    term: string,
    withSound: boolean,
}

const request = util.promisify(requestModule)

export async function queryJisho(args: JishoArgs) {
    const params = { keyword: args.term }

    const searchPage = args.withSound && request({
        uri:     JISHO_SEARCH_URI + encodeURIComponent(args.term),
        method:  'GET',
        timeout: JISHO_SEARCH_TIMEOUT_MS,
    })

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
            japanese.audio = []
        }
    }

    if (searchPage) {
        try {
            const result = await searchPage
            const dom = new JSDOM(result.body)
            const doc = dom.window.document
            for (const audioEl of doc.querySelectorAll('audio')) {
                const parent = audioEl.closest('div.concept_light-wrapper')
                if (parent) {
                    const textEl = parent.querySelector('span.text')
                    const furiganaEl = parent.querySelector('span.furigana')
                    if (textEl && furiganaEl) {

                        // The furigana element contains one span per each
                        // character in the original text. Some spans will be
                        // empty if the related text segment is kana.
                        //
                        // Note that none of the Japanese characters fall under
                        // the UTF-16 surrogate pairs range.

                        const furigana: string[] = []
                        const text = (textEl.textContent || '').trim()

                        furiganaEl.querySelectorAll('span').forEach(it => {
                            furigana.push((it.textContent || '').trim())
                        })

                        if (furigana.length !== text.length) {
                            const lenA = furigana.length
                            const lenB = text.length
                            console.error(
                                `[WARN] Jisho furigana for ${text} does not match length (${lenA} !== ${lenB})`)
                        } else {
                            for (let i = 0; i < text.length; i++) {
                                furigana[i] = furigana[i] || text[i]
                            }
                        }

                        const reading = furigana.join('')
                        const audio: string[] = []
                        audioEl.querySelectorAll('source').forEach(el => {
                            const src = el.getAttribute('src')
                            if (src) {
                                audio.push(src)
                            }
                        })

                        for (const it of data) {
                            for (const jp of it.japanese) {
                                if (jp.reading === reading && jp.word === text) {
                                    jp.audio.push(...audio)
                                }
                            }
                        }
                    }
                }
            }
        } catch (err) {
            console.error(`[WARN] failed to load audio data from Jisho:`, err)
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

    /** Audio URLs for this term. Only available when loading audio. */
    audio: string[],
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
