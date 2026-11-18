<p align="center">
  <img src="OBJSI_DARK.png" alt="Objective Superintelligence" width="320">
</p>

# Objective-Superintelligence

An agent for [Diverge](https://github.com/divergenetwork/Diverge) that
answers a structured decision by asking other agents the same decision
and averaging what they say, each weighed by a weight you give it.

Two container images, built from the two folders here:

| image | what it is |
|---|---|
| [`Objective-Superintelligence-Agent`](Objective-Superintelligence-Agent/.diverge/DIVERGE.md) | The agent. Made with a list of agents and a weight for each. Takes structured decisions only, no chat. |
| [`Objective-Superintelligence-Tool`](Objective-Superintelligence-Tool/.diverge/DIVERGE.md) | The agent's one dependency tool, with no arguments. To the agent, it puts a decision to the agents the agent names and returns each one's answer, or that it failed. To any other agent that declares it, it serves `get` and `vote`: the agent's queue of decisions to vote on, and its vote. |

## How a decision is answered

1. You send the agent a structured decision: a prompt and questions of
   type `noul`, `choice` or `score`.
2. The agent calls the tool, which puts the same decision to every
   agent in the list and returns each one's answers. A structured
   decision agent is sent the decision. A voter finds it in its queue
   in the tool, and votes there.
3. Agents that failed, or whose answers do not fit the questions, are
   left out.
4. The agent answers each question with the weighted average of the
   rest: probabilities only. The daemon adds the winning `choice` and
   the `score` when it logs the answer.

## Making one

The agent's template arguments name the tool's image, and list the
agents to ask, each by its kind, the id of the template it was made
from and its index, with its weight:

```json
{"tool": {"digest": "sha256:<the tool image's digest>"},
 "agents": [
  {"type": "structured_decision", "template": "<agent template id>", "index": 1, "weight": 3},
  {"type": "voter", "template": "<agent template id>", "index": 2, "weight": 1}
]}
```

`tool` is the Objective-Superintelligence-Tool image that goes with
the agent image: see "Images" below. A tool built for another agent
image does not serve the agent, and every decision fails.

There are two kinds:

- A `structured_decision` agent accepts structured decisions itself.
  It may be another Objective-Superintelligence agent.
- A `voter` is any agent that declares the Objective-Superintelligence
  tool as a dependency, with `{}` arguments, lazy and global. It is
  told when a decision waits for it, reads it with the tool's `get`,
  and answers with `vote`: one pick per question.

A weight is any number greater than zero. The tool is declared by the
agent itself, from the image `tool` names, with no arguments; the
agent names its agents on each call. You do not create or attach it.

A structured decision agent on another daemon is listed with the route
to that daemon, `relay`: the ids of the remote daemons to go through, the first one
this daemon's (`daemons list`), as any relayed request names them. Its
template and index are then the ones it has on that daemon:

```json
{"type": "structured_decision", "template": "<agent template id>", "index": 1, "weight": 1, "relay": ["<daemon id>"]}
```

An agent on this daemon has no `relay`. An empty one is refused. A
voter is always on this daemon, and a `relay` on one is refused.

## Images

Each image holds the other's digest in some way, and an image cannot
hold a digest that is computed over itself. So the two are tied in one
direction only:

- The tool's image holds the agent's digest. The tool serves
  `structured_decisions` to that agent image alone.
- The agent's image holds nothing of the tool. The tool's digest is
  given in each agent template's arguments, as `tool`.

The pair to use together:

| image | digest |
|---|---|
| `Objective-Superintelligence-Agent` | not built yet |
| `Objective-Superintelligence-Tool` | not built yet |

A digest is only usable on a provider whose Store holds that image, or
that can fetch it. On another host, build both.

## Building

Each folder is a Rust crate with its own `Containerfile`, whose build
context is that folder. Build the agent first, with Diverge itself:

```bash
diverge run images build '{"provider": <provider identity>,
  "containerfile": "<the text of Objective-Superintelligence-Agent/Containerfile>",
  "context": {"volume": "<a volume holding this repository>", "path": ["Objective-Superintelligence-Agent"]}}'
# … {"built":{"digest":"sha256:…","bytes":…}}
```

Put that digest in `Objective-Superintelligence-Tool/src/caller.rs`, as
`SUPERINTELLIGENCE_AGENT`, and build the tool the same way with
`"path": ["Objective-Superintelligence-Tool"]`. The agent's digest is
what an agent template names as its image, and the tool's digest is
what its arguments name as `tool`.

As committed, `SUPERINTELLIGENCE_AGENT` is a placeholder that no image
has, so a tool built from it serves `structured_decisions` to no one.
