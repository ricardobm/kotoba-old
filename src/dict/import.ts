/**
 * Support for importing Yomichan dictionary format.
 */

import * as fs from 'fs'
import * as path from 'path'
import * as JSZip from 'jszip'
import { promisify } from 'util'

import { Entry, EntrySource } from './dict'

/**
 * Contains the raw data imported from an Yomichan dictionary folder.
 */
export class ImportedDict {

    /** Dictionary name. */
    title: string

    /** Dictionary format (expected `3`). */
    format: number

    /** Dictionary revision tag. */
    revision: string

    /** Unused */
    sequenced: boolean

    /** Source path for the dictionary. */
    path: string

    /** Dictionary terms. */
    terms: ImportedTerm[]

    /** Dictionary kanjis. */
    kanjis: ImportedKanji[]

    /** Dictionary tags. */
    tags: ImportedTag[]

    /** Frequency metadata for terms. */
    termMeta: ImportedMeta[]

    /** Frequency metadata for kanjis. */
    kanjiMeta: ImportedMeta[]

    /** Return all terms in the dictionary as entries. */
    getEntries(): Entry[] {
        return this.terms.map(x => EntryFromImportedTerm(this.title, x))
    }

    count(acc: any) {
        const cur = acc || {}
        return {
            terms:     this.terms.length     + (cur.terms     | 0),
            kanjis:    this.kanjis.length    + (cur.kanjis    | 0),
            tags:      this.tags.length      + (cur.tags      | 0),
            termMeta:  this.termMeta.length  + (cur.termMeta  | 0),
            kanjiMeta: this.kanjiMeta.length + (cur.kanjiMeta | 0),
        }
    }

    banks() {
        return [
            `terms:${this.terms.length}`,
            `kanjis:${this.kanjis.length}`,
            `tags:${this.tags.length}`,
            `meta-terms:${this.termMeta.length}`,
            `meta-kanjis:${this.kanjiMeta.length}`,
        ].join(' ')
    }

    dumpTags() {
        const tags = ([] as ImportedTag[]).concat(this.tags)
        tags.sort((a, b) => a.name.localeCompare(b.name))
        console.log(`\n=> Tags for ${this.title}:`)
        console.log(tags.map(x => `   - ${x.name} (${x.category}/${x.order}/${x.score}) ${x.notes}`).join('\n'))
    }
}

/**
 * Dictionary entry for a term.
 *
 * Each entry contains a single definition for the term given by `expression`.
 * The definition itself consists of one or more `glossary` items.
 */
export type ImportedTerm = {

    /** Term expression. */
    expression: string

    /** Kana reading for this term. */
    reading: string

    /**
     * Tags for the term definitions.
     */
    definitionTags: string[]

    /**
     * Rules that affect the entry inflections. Those are also tags.
     *
     * One of `adj-i`, `v1`, `v5`, `vk`, `vs`.
     *
     * - `adj-i` adjective (keiyoushi)
     * - `v1`    Ichidan verb
     * - `v5`    Godan verb
     * - `vk`    Kuru verb - special class (e.g. `いって来る`, `來る`)
     * - `vs`    noun or participle which takes the aux. verb suru
     */
    rules: string[]

    /** Score for this entry. Higher values have precedence. */
    score: number

    /** Definition for this entry. */
    glossary: string[]

    /** Sequence number for this entry in the dictionary. */
    sequence: number

    /** Tags for the main term. */
    termTags: string[]
}

/**
 * Creates an `Entry` from an `ImportedTerm`.
 */
export function EntryFromImportedTerm(origin: string, data: ImportedTerm): Entry {
    const entry: Entry = {
        source:         EntrySource.Import,
        origin:         origin,
        expression:     data.expression,
        reading:        data.reading,
        score:          data.score,
        tags:           uniqueStrings(data.termTags.concat(data.rules.concat())),
        extra_forms:    [],
        extra_readings: [],
        english: [
            {
                glossary: data.glossary,
                tags:     data.definitionTags,
                info:     [],
                links:    [],
            }
        ]
    }
    return entry
}

function uniqueStrings(src: string[]): string[] {
    const dup: {[index: string]: boolean} = {}
    return src.filter(item => {
        if (dup[item]) {
            return false
        }
        dup[item] = true
        return true
    })
}

/** Dictionary entry for a kanji. */
export type ImportedKanji = {

    /** Kanji character. */
    character: string

    /** Onyomi (chinese) readings for the Kanji. */
    onyomi: string[]

    /** Kunyomi (japanese) readings for the Kanji. */
    kunyomi: string[]

    /** Tags for the Kanji. */
    tags: string[]

    /** Meanings for the kanji. */
    meanings: string[]

    /**
     * Additional kanji information. The keys in `stats` are further detailed
     * by the dictionary tags.
     */
    stats: { [key: string]: string }
}

/**
 * Tag for an `ImportedKanji` or `ImportedTerm`. For kanji dictionary.
 *
 * For a `ImportedKanji`, this is also used to describe the `stats` keys.
 */
export type ImportedTag = {

    /** Name to reference this tag. */
    name: string

    /** Category for this tag. This can be used to group related tags. */
    category: string

    /**
     * Sort order for this tag (less is higher). This has higher priority than
     * the name.
     */
    order: number

    /** Description for this tag. */
    notes: string

    /** Unused */
    score: number
}

/**
 * Frequency metadata for kanjis or terms.
 */
export type ImportedMeta = {

    /** Kanji or term. */
    expression: string

    /** Always `"freq"`. */
    mode: string

    /** Metadata value. */
    data: number
}

const readFile = promisify(fs.readFile)
const readdir  = promisify(fs.readdir)

/**
 * Load an Yomichan dictionary given the base directory.
 *
 * @param basePath  The directory containing `index.json` and other data files.
 */
export async function importDict(basePath: string): Promise<ImportedDict> {

    //
    // Read from the input directory or .zip file:
    //

    const filterBanks = (name: string) => name !== 'index.json' && path.extname(name).toLowerCase() === '.json'

    let summary: any
    let files: Array<{name: string, json: any}>
    if (path.extname(basePath).toLowerCase() === '.zip') {
        const data = await readFile(basePath)
        const zip =  await JSZip().loadAsync(data)
        const index = zip.file('index.json')
        if (!index) {
            throw new Error(`'index.json' not found in ${basePath}`)
        }

        summary = JSON.parse(await index.async('text'))
        files = await Promise.all(
            zip.filter(filterBanks).map(async file => {
                return {
                    name: file.name,
                    json: JSON.parse(await file.async('text')),
                }
            })
        )
    } else {
        summary = JSON.parse(await readFile(path.join(basePath, 'index.json'), 'utf-8'))
        files = await Promise.all(
            (await readdir(basePath)).filter(filterBanks).map(async name => {
                const fullPath = path.join(basePath, name)
                return {
                    name: name,
                    json: JSON.parse(await readFile(fullPath, 'utf-8')),
                }
            })
        )
    }

    //
    // Parse data:
    //

    const out = new ImportedDict()

    out.title =     summary.title
    out.format =    summary.format
    out.revision =  summary.revision
    out.sequenced = summary.sequenced

    out.path = basePath

    out.terms =  []
    out.kanjis = []
    out.tags =   []

    out.kanjiMeta = []
    out.termMeta =  []

    const flagged: {[kind:string]: boolean} = {}
    for (const it of files) {
        const kind = it.name.replace(/(_bank(_\d+)?)?\.json$/, '')
        const data = it.json as any[][]
        switch (kind) {
            case 'term':
                for (const it of data) {
                    const [expression, reading, definitionTags, rules, score, glossary, sequence, termTags] = it
                    out.terms.push({
                        expression,
                        reading,
                        definitionTags: strToList(definitionTags),
                        rules: strToList(rules),
                        score,
                        glossary,
                        sequence,
                        termTags: strToList(termTags),
                    })
                }
                break
            case 'kanji':
                for (const it of data) {
                    const [character, onyomi, kunyomi, tags, meanings, stats] = it
                    out.kanjis.push({
                        character,
                        onyomi:  strToList(onyomi),
                        kunyomi: strToList(kunyomi),
                        tags:    strToList(tags),
                        meanings,
                        stats,
                    })
                }
                break
            case 'tag':
                for (const it of data) {
                    const [name, category, order, notes, score] = it
                    out.tags.push({
                        name,
                        category,
                        order,
                        notes,
                        score
                    })
                }
                break
            case 'kanji_meta':
                out.kanjiMeta.push(...data.map(toMeta))
                break
            case 'term_meta':
                out.termMeta.push(...data.map(toMeta))
                break
            default:
                if (!flagged[kind]) {
                    console.error(`[WARN] unrecognized bank kind '${kind}' in ${basePath} (IMPORT)`)
                    flagged[kind] = true
                }
                break
        }
    }

    // Sort by sequence
    out.terms.sort((a, b) => a.sequence - b.sequence)

    return out

    function strToList(s: any): string[] {
        return s ? (s as string).split(/\s+/) : []
    }

    function toMeta(row: any): ImportedMeta {
        const [expression, mode, data] = row
        return { expression, mode, data }
    }

}
