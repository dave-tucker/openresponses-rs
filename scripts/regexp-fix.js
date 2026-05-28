// Bun on Linux (JavaScriptCore) rejects {n,m} quantifiers that are "out of order":
//   1. m > 65535 (JSC internal limit), e.g. {0,10485760} from zod v4 maxLength
//   2. n > m (logically invalid), e.g. {1,0} from degenerate kubb-generated schemas
// Proactively rewrite these before handing the pattern to the native engine.
const _RegExp = globalThis.RegExp;

function fixQuantifiers(source) {
    return source.replace(/\{(\d+),(\d+)\}/g, (match, min, max) => {
        const n = Number(min);
        const m = Number(max);
        if (n > m) {
            // Logically impossible quantifier — substitute with {0,0} (matches empty)
            return "{0,0}";
        }
        if (m > 65535) {
            if (n === 0) return "*";
            if (n === 1) return "+";
            return `{${n},}`;
        }
        return match;
    });
}

class PatchedRegExp extends _RegExp {
    constructor(pattern, flags) {
        if (typeof pattern === "string") {
            pattern = fixQuantifiers(pattern);
        }
        super(pattern, flags);
    }
}

globalThis.RegExp = PatchedRegExp;
