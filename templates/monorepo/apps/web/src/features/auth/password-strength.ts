/**
 * Advisory password strength, from the same algorithm family as the server
 * (zxcvbn, score 0-4). The server rejects scores below 2 and feeds it the
 * account's email and display name as known inputs; we do the same so the
 * meter agrees with it more often than not. Only the server is authoritative.
 *
 * The dictionaries are large, so they load on first use, not with the app.
 */
export const SERVER_MIN_SCORE = 2

export type Score = 0 | 1 | 2 | 3 | 4

export type Strength = {
  score: Score
  label: string
  warning?: string
  suggestion?: string
  /** Would the server's threshold accept it (score >= 2)? */
  acceptable: boolean
}

const LABELS = ["Very weak", "Weak", "Fair", "Good", "Strong"] as const

export function strengthLabel(score: Score) {
  return LABELS[score]
}

type Checker = (password: string, inputs: string[]) => Strength

let checkerPromise: Promise<Checker> | undefined

function loadChecker(): Promise<Checker> {
  checkerPromise ??= (async () => {
    const [{ ZxcvbnFactory }, common, en] = await Promise.all([
      import("@zxcvbn-ts/core"),
      import("@zxcvbn-ts/language-common"),
      import("@zxcvbn-ts/language-en"),
    ])
    const factory = new ZxcvbnFactory({
      translations: en.translations,
      graphs: common.adjacencyGraphs,
      dictionary: { ...common.dictionary, ...en.dictionary },
    })
    return (password, inputs) => {
      const result = factory.check(password, inputs)
      const score = result.score
      return {
        score,
        label: strengthLabel(score),
        warning: result.feedback.warning ?? undefined,
        suggestion: result.feedback.suggestions[0],
        acceptable: score >= SERVER_MIN_SCORE,
      }
    }
  })()
  return checkerPromise
}

/** Empty and blank known inputs are dropped, like a form with nothing typed yet. */
export function cleanKnownInputs(inputs: (string | null | undefined)[]) {
  return inputs
    .map((value) => value?.trim())
    .filter((value): value is string => Boolean(value))
}

export async function checkPasswordStrength(
  password: string,
  knownInputs: (string | null | undefined)[] = []
): Promise<Strength> {
  const check = await loadChecker()
  return check(password, cleanKnownInputs(knownInputs))
}
