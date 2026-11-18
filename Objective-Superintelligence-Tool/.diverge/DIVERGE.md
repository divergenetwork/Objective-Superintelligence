---
{
  "type": "tool",
  "name": "Objective-Superintelligence-Tool",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments the tool is made with.\n\nNone: the tool is the same for every agent that declares it, and\nwhat it serves is decided by who calls, not by how it was made.\n`{}` is the one value; any member is refused, so a misspelled\nsetting is never silently ignored.\n\n# How it runs\n\nAs a dependency tool. What it serves depends on its caller, as the\ndaemon attests it on every request: an\nObjective-Superintelligence agent is served `structured_decisions`,\nwhich puts a decision to the agents the call lists; every other\ncaller is served `sum`. See [`caller`](crate::caller).",
    "type": "object",
    "additionalProperties": false
  }
}
---
# Objective-Superintelligence-Tool

The dependency tool of the Objective-Superintelligence agent. It takes
no arguments, and serves one of two sets of tools by who calls it.

## Arguments

```json
{}
```

None. Any member is refused.

## Who gets what

Every request the tool receives carries its caller, as the daemon
attests it. The tool reads that, and nothing the request says of
itself:

| caller | served |
|---|---|
| An agent whose image is the Objective-Superintelligence agent's, whatever its template, index or name | `structured_decisions` only |
| Anyone else: any other agent, or a client over a connect | `sum` only |

A tool of the other mode is no tool to the caller: listing does not
show it, and calling it is refused as a tool that does not exist.

## `structured_decisions`

| parameters | does |
|---|---|
| `agents`, `content`, `questions` | sends the decision to every agent listed and returns each one's outcome |

`agents` lists the agents asked, at least one:

```json
{"agents": [
  {"template": "<agent template id>", "index": 1},
  {"template": "<agent template id>", "index": 2, "relay": ["<daemon id>"]}
]}
```

Each is named by its template and index. An agent on another daemon
carries `relay`, the route to that daemon: the first id is this
daemon's record of a remote daemon, and each id after it is the record
the daemon before it keeps of the next. `template` and `index` are
then the agent's on the daemon the route ends at. An agent on this
daemon has no `relay`; an empty one, or `null`, is refused.

`content` and `questions` are a structured decision exactly as
`agents message` takes one: content blocks, and questions of type
`noul`, `choice` or `score`, each with its own `id`.

The result has one entry per agent, in the order `agents` lists them:

```json
{"results": [{"answer": {"<question id>": {"noul": 0.25}}}, "failure"]}
```

- `{"answer": {…}}`: the agent's answers by question id, as the agent
  gave them: a `noul`, or `probabilities` (by option for a choice, by
  level for a score). The `choice` and `score` the agent's daemon
  computed for its log are left out. Nothing is checked against the
  questions.
- `"failure"`: the agent gave no answer. It refused the decision, could
  not be reached, its route could not be taken, it never took the
  decision up, or it went idle without answering.
  Which of these is not said.

Every agent is asked at once. Nothing times a slow agent out: the call
returns when every agent has answered or its run has ended.

## `sum`

| parameters | does |
|---|---|
| `a`, `b` | adds the two numbers |

```json
{"a": 2, "b": 3}
```

The result is `{"sum": 5}`.

## Permissions

Declared at registration:

```json
[
  {"agents": {"actions": ["message"], "within": "any"}},
  {"daemons": {"actions": ["relay"], "within": "any"}}
]
```

The tool takes no arguments, so it cannot know at registration which
agents a call will list, and each grant reaches `"any"`.

- `message` over agents reaches the agents on this daemon.
- `relay` over remote daemons reaches the agents on others. This
  daemon judges only that the tool may relay to the first daemon of a
  route, and the daemon the route ends at judges the message, as the
  account this daemon's record names. `relay` is one action for every
  use of a remote daemon, so it allows any request relayed there, not
  messages only.

Nothing else. The tool takes no secret and keeps no state.
