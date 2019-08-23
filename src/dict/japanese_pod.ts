import * as requestModule from 'request'
import * as util from 'util'
import { JSDOM } from 'jsdom'

const request = util.promisify(requestModule)

/**
 * Arguments to `queryJapanesePod`.
 */
export type JapanesePodArgs = {

    /** Term to lookup. */
    term: string,

    /** If `true`, only look the 20,000 most common words. */
    common?: boolean,

    /** If `true`, allow vulgar terms in the results. */
    vulgar?: boolean,

    /** If `true`, match the start of a word instead of exactly. */
    starts?: boolean,
}

/**
 * Entry from `japanesepod101.com` results.
 */
export type JapanesePodEntry = {

    /** Main japanese term. */
    term: string,

    /** Kana reading of the term. */
    kana: string,

    /** Audio URLs. */
    audio: string[],

    /** English definition for the term. */
    english: string,

    /**
     * Additional information to append to the english definition.
     *
     * Appears italicized in grey.
     */
    english_info: string[],
}

/**
 * Query `japanesepod101.com` dictionary.
 */
export async function queryJapanesePod(args: JapanesePodArgs) {
    const params: {[key:string]: string} = {
        post: 'dictionary_reference',
        match_type: args.starts ? 'starts' : 'exact',
        search_query: args.term,
    }
    if (args.vulgar) {
        params.vulgar = 'true'
    }
    if (args.common) {
        params.common = 'true'
    }

    const response = await request({
        method: 'POST',
        uri:    'https://www.japanesepod101.com/learningcenter/reference/dictionary_post',
        form:   params,
    })

    const results: JapanesePodEntry[] = []

    const dom = new JSDOM(response.body)
    const doc = dom.window.document
    const rows = doc.querySelectorAll('div.dc-result-row')
    rows.forEach(row => {
        const termEl = row.querySelector('span.dc-vocab')
        const term = (termEl && termEl.textContent || '').trim()

        const kanaEl = row.querySelector('span.dc-vocab_kana')
        const kana = (kanaEl && kanaEl.textContent || '').trim()

        const audio: string[] = []
        row.querySelectorAll('div.di-player:first-of-type audio > source').forEach(el => {
            const src = el.getAttribute('src')
            src && audio.push(src)
        })

        const englishEl = row.querySelector('span.dc-english')
        const english_info: string[] = []
        if (englishEl) {
            const toRemove: Element[] = []
            englishEl.querySelectorAll('span.dc-english-grey').forEach(el => {
                const info = (el.textContent || '').trim()
                english_info.push(info)
                toRemove.push(el)
            })
            for (const el of toRemove) {
                el.remove()
            }
        }

        const english = (englishEl && englishEl.textContent || '').trim()

        if (term || kana) {
            results.push({ term, kana, audio, english, english_info })
        }
    })

    return results
}
