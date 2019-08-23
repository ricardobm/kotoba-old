import * as requestModule from 'request'
import * as util from 'util'
import { JSDOM } from 'jsdom'

const FORVO_SEARCH = 'https://forvo.com/search/{TERM}/ja'
const FORVO_SEARCH_TIMEOUT_MS = 5000

/**
 * Entry in the result of Forvo queries.
 */
export type ForvoEntry = {
    /** Main term for this entry. */
    text: string,

    /**
     * Secondary terms for this entry, when available.
     *
     * This can be the reading (if the main term is kanji), or can also be
     * the kanji if the main term is kana.
     */
    terms: string[],

    /** Is this entry a phrase? */
    phrase: boolean,

    /** Mp3 media URLs. */
    mp3: string[],

    /** Ogg media URLs. */
    ogg: string[],

    /**
     * URI for the specific word page. This is only available when loading the
     * search.
     */
    wordURI?: string,
}

const request = util.promisify(requestModule)

/**
 * Search for pronunciations of the term in Forvo.
 */
export async function queryForvo(term: string) {
    return scrapePage(FORVO_SEARCH.replace('{TERM}', encodeURIComponent(term)))
}

/**
 * Loads additional pronunciations for a given entry.
 */
export async function queryForvoExtra(entry: ForvoEntry) {
    if (!entry.wordURI) {
        return []
    }
    return scrapePage(entry.wordURI, entry.text, entry.terms)
}

export async function scrapePage(uri: string, text?: string, terms?: string[]) {
    const result = await request({
        uri,
        method:  'GET',
        timeout: FORVO_SEARCH_TIMEOUT_MS,
    })
    const entries = scrapeAudioData(result.body)
    if (text || (terms && terms.length)) {
        for (const it of entries) {
            it.text = it.text || text || ''
            if (terms) {
                it.terms.push(...terms)
            }
        }
    }
    return entries
}

// DOM format
// ==========
//
// On the main results page:
//
//     <LI>
//         <SPAN id="play_123" class="play" onclick="...">XYZ pronunciation</SPAN>
//         <A class="word" HREF="https://forvo.com/word/XYZ/#ja">XYZ</A>
//     </LI>
//
// The word results page (e.g. `https://forvo.com/word/家_(うち)/#ja`) has the same
// format as above, minus the link.
//
// The onclick handler for the `span.play` is as follows:
//
//     Play(123,'Base64 MP3-A','Base64 OGG-A',false,'Base64 MP3-B','Base64 OGG-B','h')
//
// The `Play` function boils down to:
//
//     function Play(id, mp3_A, ogg_A, is_auto_play, mp3_B, ogg_B, mode) {
//         mp3_A = "https://audio00.forvo.com/mp3/" + base64_decode(mp3_A),
//         ogg_A = "https://audio00.forvo.com/ogg/" + base64_decode(ogg_A);
//         mp3_B = mp3_B && "https://audio00.forvo.com/audios/mp3/" + base64_decode(mp3_B);
//         ogg_B = ogg_B && "https://audio00.forvo.com/audios/ogg/" + base64_decode(ogg_B);
//         createAudioObject(id, mp3_A, ogg_A, is_mobile(), is_auto_play, mp3_B, ogg_B, mode || "l")
//     }
//
//     function createAudioObject(id, mp3_A, ogg_A, mobile, is_auto_play, mp3_B, ogg_B, mode) {
//         let audio = document.createElement("audio");
//         if (mode == "h") {
//             if (mp3_B) add_src(audio, "audio/mp3", mp3_B)
//             if (ogg_B) add_src(audio, "audio/ogg", ogg_B)
//         }
//         if (mp3_A) add_src(audio, "audio/mp3", mp3_A)
//         if (ogg_A) add_src(audio, "audio/ogg", ogg_A)
//         audio.play()
//     }
//
// There is also phrases, which have a slightly different handler, but overall
// the same logic:
//
//     PlayPhrase(123,'Base64 MP3','Base64 OGG')
//
// The audio URLs for phrases are:
//
//     https://audio00.forvo.com/phrases/mp3/
//     https://audio00.forvo.com/phrases/ogg/

function scrapeAudioData(html: string) {

    const dom = new JSDOM(html)
    const doc = dom.window.document
    const play = doc.querySelectorAll('li > span.play')

    const entries: ForvoEntry[] = []

    play.forEach(el => {
        const parent = el.parentNode!
        const action = el.getAttribute('onclick')
        const target = parent.querySelector(':scope > a')
        if (action && /^Play(Phrase)?\(/i.test(action)) {
            const { phrase, mp3, ogg } = extractArgs(action)
            if (mp3.length + ogg.length > 0) {
                const entry: ForvoEntry = { text: '', terms: [], phrase, mp3, ogg }
                if (target) {
                    const uri = target.getAttribute('href')
                    if (uri) {
                        entry.wordURI = uri
                    }

                    entry.text = (target.textContent || '').trim()

                    const staP = `\\(（〈『【「`
                    const endP = `\\)）〉』】」`

                    const extraTermsRE = new RegExp(`(?!^)\\s*[${staP}][^${endP}]*[${endP}]$`)
                    const match = extraTermsRE.exec(entry.text)
                    if (match) {
                        entry.text = entry.text.slice(0, match.index).trim()
                        entry.terms.push(...
                            match[0].trim()
                                .replace(new RegExp(`[${staP}${endP}]`, 'g'), '')
                                .split(/\s*[,;；，、]\s*/)
                                .map(x => x.trim())
                                .filter(x => !!x))
                    }
                }
                entries.push(entry)
            }
        }
    })

    return entries
}

function extractArgs(play: string) {
    const phrase = /^PlayPhrase/i.test(play)
    const args = play.trim()
        .replace(/^Play(Phrase)?\((\d+\s*,\s*)?/, '')
        .replace(/\s*;\s*return\s*false(;)?$/, '')
        .replace(/(\s*,\s*['"]\w['"])?\s*\)$/, '')
        .replace(/\s*,\s*(false|true)\s*/g, '')
        .split(/\s*,\s*/)
        .map(arg => {
            try {
                const value = arg.replace(/["']/g, '').trim()
                if (value) {
                    const buffer = Buffer.from(value, 'base64')
                    return buffer.toString('utf-8')
                }
            } catch (e) {
                console.error(e)
            }
            return ''
        })

    if (phrase) {
        args.length = Math.min(args.length, 2)
        rebase(0, 'https://audio00.forvo.com/phrases/mp3/')
        rebase(1, 'https://audio00.forvo.com/phrases/ogg/')
    } else {
        args.length = Math.min(args.length, 4)
        rebase(0, 'https://audio00.forvo.com/mp3/')
        rebase(1, 'https://audio00.forvo.com/ogg/')
        rebase(2, 'https://audio00.forvo.com/audios/mp3/')
        rebase(3, 'https://audio00.forvo.com/audios/ogg/')
    }

    // The "B" media files (at the end) appear to be encoded in a smaller size,
    // so we prefer those.
    const uri = args.filter(x => !!x).reverse()

    const mp3 = uri.filter(x => /\.mp3$/i.test(x))
    const ogg = uri.filter(x => /\.ogg$/i.test(x))

    return { phrase, mp3, ogg }

    function rebase(index: number, baseURL: string) {
        if (args[index]) {
            args[index] = baseURL + args[index]
        }
    }
}
