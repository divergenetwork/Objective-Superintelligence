---
{
  "type": "agent",
  "name": "Objective-Superintelligence-Agent",
  "chat": false,
  "structured_decisions": {
    "prompt_modalities": {
      "image": true,
      "audio": true,
      "file": true
    },
    "instruction_modalities": {
      "image": true,
      "audio": true,
      "file": true
    },
    "choice_modalities": {
      "image": true,
      "audio": true,
      "file": true
    }
  },
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments the agent is made with: the agents every decision is\nput to, each with the weight its answer is averaged by.",
    "type": "object",
    "properties": {
      "agents": {
        "description": "The agents asked. At least one.",
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
        "description": "One agent asked: the template it was made from and its index among\nthe agents made from that template, which together name it once\nand for all on the daemon that holds it; that daemon, where it is\nanother; and its weight.",
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
          },
          "weight": {
            "description": "How much this agent's answer counts beside the others': a\nnumber greater than zero. Only the proportions matter: weights\nof 1 and 3 average as weights of 0.25 and 0.75 do.",
            "type": "number",
            "format": "double",
            "exclusiveMinimum": 0
          }
        },
        "additionalProperties": false,
        "required": [
          "template",
          "index",
          "weight"
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
# Objective-Superintelligence: the agent

Answers a structured decision with the weighted average of other
agents' answers. It takes structured decisions only: it has no chat.

## Arguments

```json
{"agents": [
  {"template": "<agent template id>", "index": 1, "weight": 3},
  {"template": "<agent template id>", "index": 2, "weight": 1},
  {"template": "<agent template id>", "index": 1, "weight": 1, "relay": ["<daemon id>"]}
]}
```

The agents asked, each by its template and index, each with a `weight`
greater than zero. At least one. Only the proportions of the weights
matter.

An agent on another daemon carries `relay`: the route to that daemon,
as every daemon request's `relay` is. The first id is this daemon's
record of a remote daemon (`daemons list`), and each id after it is
the record the daemon before it keeps of the next. `template` and
`index` are then the agent's on the daemon the route ends at. An agent
on this daemon has no `relay`: an empty one, or `null`, is refused, so
that one list of agents is one template.

## What it does with a decision

1. Echoes the decision.
2. Calls its dependency tool, the Objective-Superintelligence tool, which
   puts the same decision to every agent listed and returns each one's
   answers, or that it failed.
3. Leaves out every agent that failed, and every agent whose answers do
   not fit the questions (a question unanswered, an answer of the wrong
   type, options or levels that do not match the question's).
4. Answers each question with the average over the agents left, each
   weighed by its weight, the weights of those left re-normalised:
   - `noul`: the average `noul`;
   - `choice`: the average probability of each option;
   - `score`: the average probability of each level.

   It names no winning option and states no score. The daemon computes
   both from the averaged probabilities as it logs the answer: `choice`
   is the option with the highest probability (the alphabetically first
   on a tie), and `score` is the sum of each level's probability times
   its position, the first level counting 0.
5. If no agent is left, or the tool could not be called, the run ends
   with a fatal notification and no answer.

## Modalities

Every modality is accepted, in the prompt, in the instructions and in
the choices: the
decision is passed on as it came. An agent listed that does not take a
modality the decision uses refuses it, and is left out as a failure.
So is an agent whose route could not be taken.

## Dependency

One, always: the Objective-Superintelligence-Tool, made with no
arguments, `{}`, lazy and global (one tool for every agent that
declares it, run while it is used), 256 MiB of memory and of disk,
with no mount and no secret. Its image is fixed in this image. The agent names
its agents (weights left out) on every call of the tool's
`structured_decisions`, which the tool serves to this agent's image
alone. The tool declares `message` over any agent and `relay` over any
remote daemon, which is all the access it has.
