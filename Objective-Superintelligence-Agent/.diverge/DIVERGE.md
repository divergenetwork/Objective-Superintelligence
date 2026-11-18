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
    "description": "The arguments the agent is made with: the image of the tool it\nasks through, and the agents every decision is put to, each with\nthe weight its answer is averaged by.",
    "type": "object",
    "properties": {
      "tool": {
        "description": "The image of the Objective-Superintelligence-Tool, which the\nagent declares as its one dependency: its digest, and where\nits bytes may be fetched, if anywhere.\n\nIt is an argument, and not a part of this image, because the\ntool's image holds this image's digest: the tool serves\n`structured_decisions` to this image alone. Were this image to\nhold the tool's digest too, each digest would be a hash over\nthe other, and neither image could be built. So the agent is\nbuilt first, the tool is built knowing the agent's digest, and\nthe tool's digest is given here. A tool built for another\nagent image serves this agent `get` and `vote` only, and every\ndecision fails.",
        "$ref": "#/$defs/Image"
      },
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
      "tool",
      "agents"
    ],
    "$defs": {
      "Image": {
        "description": "An image, by its manifest digest and the references under which\nits bytes may be fetched.",
        "type": "object",
        "properties": {
          "digest": {
            "description": "The manifest digest, `<algorithm>:<hex>`.",
            "type": "string"
          },
          "references": {
            "description": "Where the bytes may be fetched, each a registry and a path\nthere. Absent when there are none.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/Reference"
            }
          }
        },
        "required": [
          "digest"
        ]
      },
      "Reference": {
        "description": "One place the bytes an image's digest names may be fetched from: a\nregistry, and the repository path there.",
        "type": "object",
        "properties": {
          "registry": {
            "description": "The registry's host: `docker.io`, `ghcr.io`,\n`registry.example.com:5000`. No scheme and no path.",
            "type": "string"
          },
          "name": {
            "description": "The repository path at that registry. No host and no tag.",
            "type": "string"
          }
        },
        "required": [
          "registry",
          "name"
        ]
      },
      "Agent": {
        "description": "One agent asked, of one of two kinds. Internally tagged by `type`,\nsnake case: `\"structured_decision\"` or `\"voter\"`. Each kind takes\nits own members and no other.\n\nEither kind is named by the template it was made from and its\nindex among the agents made from that template, which together\nname it once and for all on the daemon that holds it, and carries\nthe weight its answer is averaged by.",
        "oneOf": [
          {
            "description": "An agent that takes structured decisions: the decision is sent\nto it as a message, and its answer is read from its log. It may\nbe on another daemon.",
            "type": "object",
            "properties": {
              "type": {
                "type": "string",
                "const": "structured_decision"
              },
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
                "description": "The daemon the agent is on, where it is not this one: the\nroute to it, as a request's `relay` is. The first ID is\nthis daemon's record of a remote daemon, and each ID after\nit is the record the daemon before it keeps of the next;\nthe agent is on the daemon the route ends at, and\n`template` and `index` are that daemon's. Left out for an\nagent on this daemon; never empty.",
                "type": "array",
                "items": {
                  "$ref": "#/$defs/DaemonId"
                },
                "minItems": 1
              },
              "weight": {
                "description": "How much this agent's answer counts beside the others': a\nnumber greater than zero. Only the proportions matter:\nweights of 1 and 3 average as weights of 0.25 and 0.75 do.",
                "type": "number",
                "format": "double",
                "exclusiveMinimum": 0
              }
            },
            "additionalProperties": false,
            "required": [
              "type",
              "template",
              "index",
              "weight"
            ]
          },
          {
            "description": "An agent that votes: it declares Objective-Superintelligence-Tool\nitself, is told that a decision waits in its queue there, reads\nit with `get` and answers it with `vote`. It is on this daemon,\nalways: the tool it votes through is this daemon's, so a voter\nhas no `relay`.",
            "type": "object",
            "properties": {
              "type": {
                "type": "string",
                "const": "voter"
              },
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
                "description": "How much this agent's answer counts beside the others': a\nnumber greater than zero. Only the proportions matter:\nweights of 1 and 3 average as weights of 0.25 and 0.75 do.",
                "type": "number",
                "format": "double",
                "exclusiveMinimum": 0
              }
            },
            "additionalProperties": false,
            "required": [
              "type",
              "template",
              "index",
              "weight"
            ]
          }
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
{"tool": {"digest": "sha256:<the tool image's digest>"},
 "agents": [
  {"type": "structured_decision", "template": "<agent template id>", "index": 1, "weight": 3},
  {"type": "structured_decision", "template": "<agent template id>", "index": 1, "weight": 1, "relay": ["<daemon id>"]},
  {"type": "voter", "template": "<agent template id>", "index": 2, "weight": 1}
]}
```

`tool` is the image of the Objective-Superintelligence-Tool: its
`digest`, and optionally `references`, the registries its bytes may be
fetched from. This document cannot state the digest. The tool's image
holds this image's digest, so it is built after this image, and after
this document. The digest to use with a published agent image is
stated in the repository's README. A tool built for another agent
image does not serve this agent, and every decision fails.

`agents` is the agents asked, each by its template and index, each with a `weight`
greater than zero. At least one. Only the proportions of the weights
matter. Each is of one of two kinds, named by `type`:

- **`structured_decision`**: an agent that takes structured decisions.
  The decision is sent to it as it came.
- **`voter`**: any agent that declares the
  Objective-Superintelligence-Tool itself, with `{}`. It is told that a
  decision waits in its queue, reads it with the tool's `get`, and
  answers with the tool's `vote`: one pick per question, counted as
  probability 1 for what it picked. It needs no support for structured
  decisions.

A structured decision agent on another daemon carries `relay`: the
route to that daemon, as every daemon request's `relay` is. The first
id is this daemon's record of a remote daemon (`daemons list`), and
each id after it is the record the daemon before it keeps of the next.
`template` and `index` are then the agent's on the daemon the route
ends at. One on this daemon has no `relay`: an empty one, or `null`,
is refused, so that one list of agents is one template.

A voter is on this daemon, always: it votes through the tool, which is
this daemon's. A `relay` on a voter is refused.

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
decision is passed on as it came. A structured decision agent that
does not take a modality the decision uses refuses it, and is left out
as a failure. So is one whose route could not be taken. A voter reads
the decision as JSON, with each image, audio or file as a part of its
own, inside the quoted string it belongs to.

## Dependency

One, always: the Objective-Superintelligence-Tool, made with no
arguments, `{}`, lazy and global (one tool for every agent that
declares it, run while it is used), 256 MiB of memory and of disk,
with no mount and no secret. Its image is the one the arguments name
as `tool`. The agent names
its agents (weights left out) on every call of the tool's
`structured_decisions`, which the tool serves to this agent's image
alone. To every other agent that declares it, the tool serves `get`
and `vote`, which is how a voter takes part. The tool declares
`message` over any agent and `relay` over any remote daemon, which is
all the access it has.
