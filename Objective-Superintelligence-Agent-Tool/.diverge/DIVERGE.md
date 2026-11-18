---
{
  "type": "tool",
  "name": "Objective-Superintelligence-Agent-Tool",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments the tool is made with: the agents every decision is\nput to.\n\n# How it runs\n\nAs a dependency tool of the Objective-Superintelligence agent, which\ndeclares it with its own agents, weights left out. The tool asks\nthe daemon for `message` over exactly the agents listed here that\nare on it, and for `relay` over exactly the remote daemons the\nroutes of the others begin at: see\n[`permissions`](crate::permissions::permissions).",
    "type": "object",
    "properties": {
      "agents": {
        "description": "The agents asked, in the order their answers are returned in.\nAt least one.",
        "type": "array",
        "items": {
          "$ref": "#/$defs/Agent"
        },
        "minItems": 1
      }
    },
    "additionalProperties": false,
    "required": [
      "agents"
    ],
    "$defs": {
      "Agent": {
        "description": "One agent asked: the template it was made from and its index among\nthe agents made from that template, which together name it once\nand for all on the daemon that holds it; and that daemon, where it\nis another.",
        "type": "object",
        "properties": {
          "template": {
            "description": "The agent's template, by id.",
            "type": "string"
          },
          "index": {
            "description": "The agent's index among the agents made from that template.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "relay": {
            "description": "The daemon the agent is on, where it is not this one: the route\nto it, as a request's `relay` is. The first ID is this\ndaemon's record of a remote daemon, and each ID after it is the\nrecord the daemon before it keeps of the next; the agent is on\nthe daemon the route ends at, and `template` and `index` are\nthat daemon's. Left out for an agent on this daemon; never\nempty.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/DaemonId"
            },
            "minItems": 1
          }
        },
        "additionalProperties": false,
        "required": [
          "template",
          "index"
        ]
      },
      "DaemonId": {
        "type": "string",
        "pattern": "^[0-9A-Za-z]{1,11}$",
        "description": "A remote daemon's record, by the hash of what it reaches and as whom: base62, the shortest form."
      }
    }
  }
}
---
# Objective-Superintelligence: the tool

Puts one structured decision to a fixed list of agents and returns what
each one answered. It is the dependency tool of
the Objective-Superintelligence agent, which does the weighing; this tool
weighs nothing.

## Arguments

```json
{"agents": [
  {"template": "<agent template id>", "index": 1},
  {"template": "<agent template id>", "index": 2, "relay": ["<daemon id>"]}
]}
```

The agents asked, each by its template and index. At least one.

An agent on another daemon carries `relay`, the route to that daemon:
the first id is this daemon's record of a remote daemon, and each id
after it is the record the daemon before it keeps of the next.
`template` and `index` are then the agent's on the daemon the route
ends at. An agent on this daemon has no `relay`; an empty one, or
`null`, is refused.

## The tool

| tool | parameters | does |
|---|---|---|
| `structured_decisions` | `content`, `questions` | sends the decision to every agent and returns each one's outcome |

`content` and `questions` are a structured decision exactly as
`agents message` takes one: content blocks, and questions of type
`noul`, `choice` or `score`, each with its own `id`.

The result has one entry per agent, in the arguments' order:

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

## Permissions

Declared at registration, from the arguments:

```json
[{"agents": {"actions": ["message"], "within": {"agents": [{"template": "<agent template id>", "index": 1}]}}}]
```

`message` over exactly the agents listed that have no `relay`. Where
any agent has one, a second grant:

```json
{"daemons": {"actions": ["relay"], "within": {"ids": ["<daemon id>"]}}}
```

`relay` over exactly the remote daemons the routes begin at, each
once. A relayed agent is in neither list by its template and index:
this daemon judges only that the tool may relay to the first daemon of
the route, and the daemon the route ends at judges the message, as the
account this daemon's record names. `relay` is one action for every
use of a remote daemon, so it allows any request relayed to those
daemons, not messages only.

Nothing else. The tool takes no secret and keeps no state.
