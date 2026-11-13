---
{
  "name": "Objective-Superintelligence",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments the tool is made with: the agents every decision is\nput to.\n\n# How it runs\n\nAs a dependency tool of Objective-Superintelligence-Agent, which\ndeclares it with its own agents, weights left out. The tool asks\nthe daemon for one thing: `message` over exactly the agents\nlisted here.",
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
        "description": "One agent asked: the template it was made from and its index among\nthe agents made from that template, which together name it once\nand for all.",
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
          }
        },
        "additionalProperties": false,
        "required": [
          "template",
          "index"
        ]
      }
    }
  }
}
---
# Objective-Superintelligence: the tool

Puts one structured decision to a fixed list of agents and returns what
each one answered. It is the dependency tool of
Objective-Superintelligence-Agent, which does the weighing; this tool
weighs nothing.

## Arguments

```json
{"agents": [{"template": "<agent template id>", "index": 1}, {"template": "<agent template id>", "index": 2}]}
```

The agents asked, each by its template and index. At least one.

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
  gave them. Nothing is checked against the questions.
- `"failure"`: the agent gave no answer. It refused the decision, could
  not be reached, never took it up, or went idle without answering.
  Which of these is not said.

Every agent is asked at once. Nothing times a slow agent out: the call
returns when every agent has answered or its run has ended.

## Permissions

Declared at registration, from the arguments:

```json
[{"agents": {"actions": ["message"], "within": {"agents": [{"template": "<agent template id>", "index": 1}]}}}]
```

`message` over exactly the agents listed, and nothing else. The tool
takes no secret and keeps no state.
