export const SECTION_MARKER = 'VITALS_SECTION'

export function splitSections(raw: string): Record<string, string> {
  const sections: Record<string, string> = {}
  const lines = raw.split('\n')
  let current: string | null = null
  const buf: string[] = []

  const flush = () => {
    if (current !== null) {
      sections[current] = buf.join('\n').trim()
      buf.length = 0
    }
  }

  for (const line of lines) {
    if (line.startsWith(`${SECTION_MARKER} `)) {
      flush()
      current = line.slice(SECTION_MARKER.length + 1).trim()
      continue
    }
    if (current !== null) {
      buf.push(line)
    }
  }
  flush()
  return sections
}
