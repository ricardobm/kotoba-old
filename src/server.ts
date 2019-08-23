import * as express from 'express'
import * as bodyParser from 'body-parser'
import * as fs from 'fs'
import * as path from 'path'

import * as dict from './dict'

const PORT     = 3001
const APP_NAME = 'Hongo'
const DATA_DIR = 'hongo-data'

// Search the file system for the user data directory, moving up the directory
// hierarchy.
//
// TODO: use a configuration instead
const userData = (() => {
    let lastPath = ''
    let curPath = path.resolve('.')
    while (curPath !== lastPath) {
        if (fs.existsSync(path.join(curPath, DATA_DIR, 'import'))) {
            return path.resolve(path.join(curPath, DATA_DIR))
        }
        lastPath = curPath
        curPath = path.resolve(path.join(curPath, '..'))
    }
    console.error('Error: could not find data directory!')
    process.exit(1)
})()!

main().catch(err => console.error('Error:', err))

async function main() {
    console.log(`Found data directory at ${userData}`)

    const start = process.hrtime()
    // await loadDicts()
    // await loadJisho()
    // await loadForvo()
    await loadJapanesePod()
    const end = process.hrtime(start)
    console.log(`\nLoading took ${(end[0] + end[1] / 1e9).toFixed(3)}s`)

    // Create the Express application
    const app = express()
    app.use(bodyParser.json())
    app.use(bodyParser.urlencoded({ extended: true }))

    app.get('/', (req, res) => {
        res.send(`Hello from ${APP_NAME}`)
    })

    app.get('/list', (req, res) => {
        res.status(200).json({
            items: [
                { id: 'A', text: 'Item A' },
                { id: 'B', text: 'Item B' },
                { id: 'C', text: 'Item C' },
            ]
        })
    })

    const server = app.listen(PORT, () => {
        console.log(`${APP_NAME} server started at http://localhost:${PORT}`)
    })

    process.once('SIGINT', shutdown)

    function shutdown() {
        console.log('Shutting down...')
        server.close(() => {
            process.exit(0)
        })
    }
}

async function loadJapanesePod() {
    const term = '家'
    const entries = await dict.queryJapanesePod({
        term: term,
        common: false,
        vulgar: true,
        starts: false,
    })

    console.log(`\n#\n# Found ${entries.length} entry(s) for ${term}\n#`)
    for (const it of entries) {
        console.log(`\n=> ${it.term} (${it.kana})`)
        for (const src in it.audio) {
            console.log(`   > ${it.audio}`)
        }

        const tags = it.english_info.join(' ')
        console.log(`   ${it.english} ${tags}`)
    }
}

async function loadForvo() {
    const term = '家'
    const entries = await dict.queryForvo(term)
    console.log(`\n#\n# Search for '${term}' #\n#`)
    showEntries(entries)

    if (entries.length > 0) {
        const entry = entries[entries.length-1]
        const extra = await dict.queryForvoExtra(entry)
        console.log(`\n## Entry: ${tagLine(entry, false)} ##`)
        showEntries(extra)
    }

    function showEntries(entries: dict.ForvoEntry[]) {
        for (const it of entries) {
            console.log(`\n=> ${tagLine(it, true)}`)
            if (it.wordURI) {
                console.log(`   URL: ${it.wordURI}`)
            }
            for (const src of it.mp3) {
                console.log(`   MP3: ${src}`)
            }
            for (const src of it.ogg) {
                console.log(`   OGG: ${src}`)
            }
        }
    }

    function tagLine(entry: dict.ForvoEntry, detailed: boolean) {
        const terms = entry.terms.length ? ` (${entry.terms.join(', ')})` : ``
        const phrase = entry.phrase && detailed ? ` [phrase]` : ``
        return `${entry.text}${terms}${phrase}`
    }
}

async function loadJisho() {
    const term = '家'
    // const term = 'お早うございます'
    // const term = '使い方'
    // const term = 'お好み焼き'
    // const term = 'お風呂に入る'
    const result = await dict.queryJisho({ term, withSound: true })
    for (const entry of result) {
        console.log(`\n\n>>> ${entry.japanese[0].word} 「${entry.japanese[0].reading}」`)

        const tags = [entry.is_common ? 'common' : 'uncommon']
            .concat(entry.jlpt).concat(entry.tags)
            .join(', ')
        if (tags) {
            console.log(`    [${tags}]`)
        }

        let counter = 0
        for (const sense of entry.senses) {
            counter++
            console.log()
            if (sense.parts_of_speech.length) {
                console.log(`    ${sense.parts_of_speech.join(', ')}`)
            }

            const number = `${counter}.`
            const indent = ' '.repeat(number.length)
            console.log(`    ${number} ${sense.english_definitions.join('; ')}`)
            if (sense.tags.length) {
                console.log(`    ${indent} ${sense.tags.join(', ')}`)
            }
            if (sense.info.length) {
                console.log(`    ${indent} ${sense.info.join(', ')}`)
            }
            if (sense.see_also.length) {
                console.log(`    ${indent} See also: ${sense.see_also.join('、')}`)
            }
            for (const link of sense.links) {
                console.log(`    ${indent} - ${link.text} (${link.url})`)
            }
            if (sense.antonyms.length) {
                console.log(`    !! ANTONYMS: ${JSON.stringify(sense.antonyms)}`)
            }
            if (sense.source.length) {
                console.log(`    !! SOURCE: ${JSON.stringify(sense.source)}`)
            }
            if (sense.restrictions.length) {
                console.log(`    !! RESTRICTIONS: ${JSON.stringify(sense.restrictions)}`)
            }
        }

        if (entry.japanese.length > 1) {
            console.log('\n    ## Other forms ##\n')
            for (const it of entry.japanese.slice(1)) {
                console.log(`    ${it.word} 「${it.reading}」`)
            }
        }

        const audio = entry.japanese.filter(x => x.audio.length)
        if (audio.length) {
            console.log('\n    ## Audio ##')
            for (const it of audio) {
                console.log(`\n    ${it.word} 「${it.reading}」`)
                for (const src of it.audio) {
                    console.log(`    > ${src}`)
                }
            }
        }
    }
    console.log(`\nFound ${result.length} entry(s) for ${term}\n`)
}

async function loadDicts() {
    const baseDir = path.join(userData, 'import')
    const dicts = await Promise.all(
        fs.readdirSync(baseDir)
            .map(it => path.join(baseDir, it))
            .map(dict.importDict)
    )
    console.log(dicts.map(
        it => `==> ${it.title} (${it.revision})\n    In ${it.path}\n    Banks: ${it.banks()}`).join('\n'))

    const count: {[key: string]: number} = dicts.reduce((acc, it) => it.count(acc), null)!
    const loaded = Object.keys(count).map(key => `${key}:${count[key]}`).join(' / ')
    console.log(`\nLoaded ${loaded}`)

    const allRules: {[key: string]: boolean} = {}
    const ruleSample: {[key: string]: string[]} = {}

    const tagMap: {[key: string]: dict.ImportedTag} = {}
    const termTags: {[key:string]: boolean} = {}
    const definitionTags: {[key:string]: boolean} = {}
    const kanjiTags: {[key:string]: boolean} = {}
    for (const dict of dicts) {
        for (const tag of dict.tags) {
            tagMap[tag.name] = tag
        }
        for (const term of dict.terms) {
            for (const tag of term.termTags) {
                termTags[tag] = true
            }
            for (const tag of term.definitionTags) {
                definitionTags[tag] = true
            }
            for (const rule of term.rules) {
                allRules[rule] = true
                ruleSample[rule] = ruleSample[rule] || []
                if (ruleSample[rule].length < 10) {
                    if (Math.random() < 0.1) {
                        ruleSample[rule].push(term.expression)
                    }
                }
            }
        }
        for (const term of dict.kanjis) {
            for (const tag of term.tags) {
                kanjiTags[tag] = true
            }
        }
    }

    function showTags(name: string, m: {[key:string]: boolean}) {
        const notFound = Object.keys(m).filter(k => !tagMap[k])
        console.log(`\n${name} tags:`)
        console.log(
            Object.keys(m).sort()
                .filter(k => tagMap[k])
                .map(k => tagMap[k])
                .map(x => `- ${x.name} (${x.category}) ${x.notes}`)
                .join('\n'))
        if (notFound.length) {
            console.error(`\nWARNING: tags not defined in ${name}: ${notFound.join(', ')}`)
        }
    }

    showTags('Term', termTags)
    showTags('Definition', definitionTags)
    showTags('Kanji', kanjiTags)

    console.log(`\nRULES: ${Object.keys(allRules).sort().join(', ')}\n`)
    for (const key of Object.keys(allRules).sort()) {
        console.log(`- ${key}: ${ruleSample[key].sort().join(', ')}`)
    }
}
