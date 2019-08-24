import * as fs from 'fs'
import * as path from 'path'
import * as util from 'util'

const fsMakeDir = util.promisify(fs.mkdir)
const fsStat    = util.promisify(fs.stat)

/**
 * Asynchronous wrapper for `fs.readFile`.
 */
export const readFile  = util.promisify(fs.readFile)

/**
 * Asynchronous wrapper for `fs.writeFile`.
 */
export const writeFile = util.promisify(fs.writeFile)

/**
 * Wrapper around `writeFile` that also attempts to create the file
 * directory if it does not exist.
 */
export async function writeFileAt(
    filename: string,
    data: any,
    options?: {
        encoding?: string | null
        mode?: number | string
        flag?: string
    } | string | undefined | null)
{
    try {
        await writeFile(filename, data, options)
    } catch (e) {
        if (e.code === 'ENOENT') {
            // Tries again, creating the directory first.
            await makeDir(path.dirname(filename))
            await writeFile(filename, data, options)
        } else {
            throw e
        }
    }
}

/**
 * Reads text from the given file. Returns null in case of errors or
 * if the file does not exist.
 */
export async function readText(filename: string) {
    try {
        return await readFile(filename, 'utf8')
    } catch (err) {
        return null
    }
}

/**
 * Writes text to the given file. Creates intermediate directories
 * if necessary.
 */
export async function writeText(filename: string, text: string) {
    try {
        await writeFile(filename, text, 'utf8')
    } catch (e) {
        if (e.code === 'ENOENT') {
            // Tries again, creating the directory first.
            await makeDir(path.dirname(filename))
            await writeFile(filename, text, 'utf8')
        } else {
            throw e
        }
    }
}

/**
 * Creates the directory, including intermediary nodes, if it does not
 * exist yet.
 */
export async function makeDir(dir: string) {
    // Makes the path absolute and parses it.
    const full   = path.resolve(dir)
    const parsed = path.parse(full)
    const parts  = full.substr(parsed.root.length).split(/[\\/]+/)

    // Iterates each part of the path and tries to create it.
    let base = parsed.root
    for (let i = 0; i < parts.length; i++) {
        const sub = path.join(base, parts[i])
        base = sub
        try {
            await fsMakeDir(sub)
        } catch (e) {
            // EEXIST is expected if the directory already exists, on
            // any other error we abort.
            if (e.code !== 'EEXIST') {
                throw e
            } else if (i === parts.length - 1) {
                // An EEXIST as we try to create the final directory
                // could also be a file, we must check for that.
                const st = await fsStat(sub)
                if (st.isFile()) {
                    throw new Error(`cannot create directory: '${sub}' is a file`)
                }
            }
        }
    }
}
