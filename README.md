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
| [`Objective-Superintelligence-Agent`](Objective-Superintelligence-Agent/DIVERGE.md) | The agent. Made with a list of agents and a weight for each. Takes structured decisions only, no chat. |
| [`Objective-Superintelligence-Tool`](Objective-Superintelligence-Tool/DIVERGE.md) | The agent's one dependency tool. Puts a decision to every one of those agents and returns each one's answer, or that it failed. |

## How a decision is answered

1. You send the agent a structured decision: a prompt and questions of
   type `noul`, `choice` or `score`.
2. The agent calls the tool, which sends the same decision to every
   agent in the list, watches each, and returns each one's answers.
3. Agents that failed, or whose answers do not fit the questions, are
   left out.
4. The agent answers each question with the weighted average of the
   rest.

## Making one

The agent's template arguments list the agents to ask, each by the id
of the template it was made from and its index, with its weight:

```json
{"agents": [
  {"template": "<agent template id>", "index": 1, "weight": 3},
  {"template": "<agent template id>", "index": 2, "weight": 1}
]}
```

A weight is any number greater than zero. The tool is declared by the
agent itself, with the same agents; you do not create or attach it.

Every agent listed must itself accept structured decisions. An agent
listed may be another Objective-Superintelligence-Agent.

## Building

Each folder is a Rust crate with its own `Containerfile`, built from
that folder. The tool is built first: the agent's `tool-image.json`
holds the tool image's digest, and is compiled into the agent.
