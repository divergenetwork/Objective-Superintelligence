---
{
  "name": "Objective-Superintelligence",
  "chat": false,
  "structured_decisions": {
    "prompt_modalities": {
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
        "description": "One agent asked: the template it was made from and its index among\nthe agents made from that template, which together name it once\nand for all, and its weight.",
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
  {"template": "<agent template id>", "index": 2, "weight": 1}
]}
```

The agents asked, each by its template and index, each with a `weight`
greater than zero. At least one. Only the proportions of the weights
matter.

## What it does with a decision

1. Echoes the decision.
2. Calls its dependency tool, Objective-Superintelligence-Tool, which
   puts the same decision to every agent listed and returns each one's
   answers, or that it failed.
3. Leaves out every agent that failed, and every agent whose answers do
   not fit the questions (a question unanswered, an answer of the wrong
   type, options or levels that do not match the question's).
4. Answers each question with the average over the agents left, each
   weighed by its weight, the weights of those left re-normalised:
   - `noul`: the average `noul`;
   - `choice`: the average probability of each option;
   - `score`: the average `score`, and the average probability of each
     level.
5. If no agent is left, or the tool could not be called, the run ends
   with a fatal notification and no answer.

## Modalities

Every modality is accepted, in the prompt and in the choices: the
decision is passed on as it came. An agent listed that does not take a
modality the decision uses refuses it, and is left out as a failure.

## Dependency

One, always: Objective-Superintelligence-Tool, made with this agent's
own agents (weights left out), transient, 256 MiB of memory and of
disk, with no mount and no secret. Its image is fixed in this image.
The tool declares `message` over exactly those agents, which is all
the access it has.
