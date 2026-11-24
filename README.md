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
| [`Objective-Superintelligence-Tool`](Objective-Superintelligence-Tool/.diverge/DIVERGE.md) | The agent's one dependency tool, made with the agent image's digest. To the agent, it puts a decision to the agents the agent names and returns each one's answer, or that it failed. To any other agent that declares it, it serves `get` and `vote`: the agent's queue of decisions to vote on, and its vote. |

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

The agent's template arguments list the agents to ask, each by its
kind, the id of the template it was made from and its index, with its
weight:

```json
{"agents": [
  {"type": "structured_decision", "template": "<agent template id>", "index": 1, "weight": 3},
  {"type": "voter", "template": "<agent template id>", "index": 2, "weight": 1}
]}
```

There are two kinds:

- A `structured_decision` agent accepts structured decisions itself.
  It may be another Objective-Superintelligence agent.
- A `voter` is any agent that declares the Objective-Superintelligence
  tool as a dependency, lazy and global, with the agent image's digest
  as its argument: `{"superintelligence_agent_digest": "sha256:…"}`.
  See "Images" below. It is
  told when a decision waits for it, reads it with the tool's `get`,
  and answers with `vote`: one pick per question.

A weight is any number greater than zero. The tool is declared by the
agent itself, which gives it its own image's digest; the agent names
its agents on each call. You do not create or attach it.

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

The agent's image holds the tool's digest, and declares the tool from
it. The tool's image holds nothing of the agent: the agent is told its
own digest when it starts, and gives it to the tool as the tool's
argument. So the tool is built first, and the agent second.

The pair to use together:

| image | digest |
|---|---|
| `Objective-Superintelligence-Tool` | not built yet |
| `Objective-Superintelligence-Agent` | not built yet |

A voter needs both: it declares the tool's image, with the agent's
digest as its argument. A digest is only usable on a provider whose
Store holds that image, or that can fetch it. On another host, build
both.

## Building

Each folder is a Rust crate with its own `Containerfile`, whose build
context is that folder. Build the tool first, with Diverge itself:

```bash
diverge run images build '{"provider": <provider identity>,
  "containerfile": "<the text of Objective-Superintelligence-Tool/Containerfile>",
  "context": {"volume": "<a volume holding this repository>", "path": ["Objective-Superintelligence-Tool"]},
  "platforms": ["linux/amd64"]}'
# … {"built":{"digest":"sha256:…","bytes":…}}
```

Put that digest in `Objective-Superintelligence-Agent/tool-image.json`,
which is compiled into the agent, and build the agent the same way
with `"path": ["Objective-Superintelligence-Agent"]`. The agent's
digest is what an agent template names as its image.

As committed, `Objective-Superintelligence-Agent/tool-image.json` names
a tool image built from an earlier revision, which took no arguments:
`sha256:4f7386f7cb6cd8cc7bab7f6e2a05b630b399de959ef24a00e48adf3948f9b1ba`.
