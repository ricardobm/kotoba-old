/**
 * Entry for the dictionary.
 */
export type Entry = {

    /**
     * Source for this entry.
     */
    source: EntrySource

    /**
     * Additional origin information for this entry (human readable). The exact
     * format and information depend on the source.
     */
    origin: string

    /**
     * Main Japanese expression for this entry.
     */
    expression: string

    /**
     * Kana reading for the expression. May be empty if `expression` itself
     * is already kana, or if not applicable.
     */
    reading: string

    /**
     * Additional Japanese forms for this entry, if available.
     */
    extra_forms: string[]

    /**
     * Readings for each `extra_forms`.
     */
    extra_readings: string[]

    /**
     * English definitions for this entry.
     */
    english: EntryEnglish[]

    /**
     * Tags that apply to the entry itself. Possible examples are JLPT
     * level, if the term is common, frequency information, etc.
     */
    tags: string[]

    /**
     * Numeric score of this entry (in case of multiple possibilities).
     *
     * Higher values appear first. This does not affect entry with different
     * origins.
     */
    score: number
}

/**
 * English meaning for an entry.
 */
export type EntryEnglish = {
    /**
     * List of glossary terms for the meaning.
     */
    glossary: string[]

    /**
     * Tags that apply to this meaning. Examples are: parts of speech, names,
     * usage, area of knowledge, etc.
     */
    tags: string[]

    /**
     * Additional information to append to the entry definition.
     */
    info: string[]

    /**
     * Related links. Those can be web URLs or other related words.
     */
    links: Link[]
}

/** Origin for a dictionary entry. */
export enum EntrySource {
    /** Entry was imported from a dictionary file. */
    Import      = 'I',
    /** Entry was imported from `jisho.org`. */
    Jisho       = 'J',
    /** Entry was imported from `japanesepod101.com`. */
    JapanesePod = 'P',
}

/** Link to related resources. */
export type Link = {
    /** URI for the linked resource. */
    uri:  string
    /** Text for this link. */
    text: string
}

export type NameMap = {
    names?: string[]
    index?: {[name: string]: number}
}

/**
 * Serializes the entries to a compacter format that can be serialized.
 */
export function serializeEntries(entries: Entry[], nameMap: NameMap) {
    nameMap = nameMap || {}
    nameMap.names = nameMap.names || []
    nameMap.index = nameMap.index || {}

    return entries.map(it => {
        return [
            it.source,
            mapName(it.origin),
            it.expression,
            it.reading,
            it.extra_forms,
            it.extra_readings,
            mapNames(it.tags),
            it.score,
            it.english.map(eng => {
                return [
                    eng.glossary,
                    mapNames(eng.tags),
                    mapNames(eng.info),
                    eng.links.map(link => [link.uri, link.text]),
                ]
            }),
        ]
    })

    function mapNames(names: string[]) {
        return names.map(mapName)
    }

    function mapName(name: string) {
        if (!name) {
            return 0
        }
        if (!nameMap.index![name]) {
            nameMap.names!.push(name)
            nameMap.index![name] = nameMap.names!.length
        }
        return nameMap.index![name]
    }
}

/**
 * Unserialize entries processed by `serializeEntries`.
 */
export function unserializeEntries(data: any[], nameMap: NameMap): Entry[] {

    return data.map(it => ({
        source:         it.source,
        origin:         mapIndex(it.origin),
        expression:     it.expression,
        reading:        it.reading,
        extra_forms:    it.extra_forms,
        extra_readings: it.extra_readings,
        tags:           mapIndexes(it.tags),
        score:          it.score,

        english: it.english.map((eng: any) => ({
            glossary: eng.glossary,
            tags:     mapIndexes(eng.tags),
            info:     mapIndexes(eng.info),

            links: eng.links.map((link: string[2]) => ({
                uri: link[0],
                text: link[1],
            })),
        })),
    }))

    function mapIndexes(indexes: number[]) {
        return indexes.map(mapIndex)
    }

    function mapIndex(index: number) {
        return index ? nameMap.names![nameMap.index![index]] : ''
    }
}
