---
{
  "type": "tool",
  "name": "Objective-Superintelligence-Tool",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments the tool is made with.\n\nNone: the tool is the same for every agent that declares it, and\nwhat it serves is decided by who calls, not by how it was made.\n`{}` is the one value; any member is refused, so a misspelled\nsetting is never silently ignored.\n\n# How it runs\n\nAs a dependency tool. What it serves depends on its caller, as the\ndaemon attests it on every request: an\nObjective-Superintelligence agent is served `structured_decisions`,\nwhich puts a decision to the agents the call lists; every other\ncaller is served `get` and `vote`, the next decision in its queue\nand its vote on one. See [`caller`](crate::caller).",
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
| Anyone else | `get` and `vote` only |

A tool of the other mode is no tool to the caller: listing does not
show it, and calling it is refused as a tool that does not exist.

`get` and `vote` work on the calling agent's own queue, which its
template and index name. A caller that is not an agent, such as a
client over a connect, is listed the two, and each call fails: it has
no queue.

## `structured_decisions`

| parameters | does |
|---|---|
| `agents`, `content`, `questions` | puts the decision to every agent listed and returns each one's outcome |

`agents` lists the agents asked, at least one. Each is of one of two
kinds, named by `type`:

```json
{"agents": [
  {"type": "structured_decision", "template": "<agent template id>", "index": 1},
  {"type": "structured_decision", "template": "<agent template id>", "index": 2, "relay": ["<daemon id>"]},
  {"type": "voter", "template": "<agent template id>", "index": 3}
]}
```

Each is named by its template and index.

- **`structured_decision`**: an agent that takes structured decisions.
  The decision is sent to it as a message, and its answer is read from
  its log. One on another daemon carries `relay`, the route to that
  daemon: the first id is this daemon's record of a remote daemon, and
  each id after it is the record the daemon before it keeps of the
  next. `template` and `index` are then the agent's on the daemon the
  route ends at. One on this daemon has no `relay`; an empty one, or
  `null`, is refused.
- **`voter`**: an agent that votes through this tool, with `get` and
  `vote`. It is on this daemon, and a `relay` on it is refused. See
  "Voters" below.

`content` and `questions` are a structured decision exactly as
`agents message` takes one: content blocks, and questions of type
`noul`, `choice` or `score`, each with its own `id`. Where a voter is
listed, the questions must be valid as a decision's are, or the call
is refused: a vote is checked against them.

The result has one entry per agent, in the order `agents` lists them:

```json
{"results": [{"answer": {"<question id>": {"noul": 0.25}}}, "failure"]}
```

- `{"answer": {…}}`: the agent's answers by question id: a `noul`, or
  `probabilities` (by option for a choice, by level for a score). A
  structured decision agent's are as it gave them, and nothing is
  checked against the questions; the `choice` and `score` its daemon
  computed for its log are left out. A voter's are its vote: 1 for
  what it picked and 0 for the rest.
- `"failure"`: the agent gave no answer. A structured decision agent
  refused the decision, could not be reached, its route could not be
  taken, it never took the decision up, or it went idle without
  answering. A voter could not be told, or went idle without voting.
  Which of these is not said.

Every agent is asked at once. Nothing times a slow agent out: the call
returns when every agent has answered or its run has ended.

## Voters

A voter is an ordinary agent that declares this tool itself, with
`{}`, so that it is served `get` and `vote`. It needs no support for
structured decisions. An agent that does not declare the tool cannot
reach it, and fails every decision it is listed in as a voter.

What happens to a voter when `structured_decisions` lists it:

1. The decision is put in the voter's queue. A decision is stored once,
   however many voters it is put to.
2. The voter is sent this chat message, word for word:

   ```
   <Objective-Superintelligence>
   You have a new structured decision in the queue.
   </Objective-Superintelligence>
   ```

3. The tool waits for the voter's vote for as long as the voter works
   on that message. The moment the vote is in, the wait ends.
4. If the voter goes idle without voting, or the message could not be
   sent, the voter's outcome is `failure`, and the decision is taken
   out of its queue.

When the call returns, the decision is deleted. A voter that has
several decisions waiting reads them one by one, oldest first.

## `get`

No parameters. It takes the next decision out of the caller's queue,
and returns it as one pretty JSON object, written across the parts of
the result:

```json
{
  "id": "0a1b2c3d",
  "content": "…",
  "questions": [
    {
      "type": "noul",
      "id": "q1",
      "instructions": "…"
    },
    {
      "type": "choice",
      "id": "q2",
      "instructions": "…",
      "options": [
        {
          "value": "red",
          "description": "…"
        }
      ]
    },
    {
      "type": "score",
      "id": "q3",
      "instructions": "…",
      "levels": [
        "…",
        "…"
      ]
    }
  ]
}
```

- `id` is the decision's id in this voter's queue: eight hex digits.
  `vote` takes it.
- `content`, a question's `instructions`, an option's `description`
  and each of a score's `levels` are lists of content blocks. Each is
  written as one quoted string, and its blocks stand inside the quotes
  as parts of their own, in order, as the decision carried them. A
  text block's text is escaped as a JSON string's is; an image, audio
  or a file is the part it was.
- Everything around the blocks (keys, quotes, commas, brackets,
  indentation) is text parts, before and after them.
- An option with no description has no `description`.

So the parts for a question whose instructions are text, an image,
then text are:

| part | holds |
|---|---|
| text | `…"instructions": "` |
| text | `Is this one better? Say \"yes\" or \"no\".` |
| image | the image |
| text | ` Ignore the border.` |
| text | `"`, and the JSON after it |

Read in order, the parts are valid JSON, with the image inside the
quotes.

- Each decision is returned once. The next call returns the one after
  it. With nothing in the queue, the result is the text `null`.

## `vote`

| parameters | does |
|---|---|
| `id`, `answers` | votes on the decision `id` |

```json
{"id": "0a1b2c3d", "answers": [
  {"type": "noul",   "id": "q1", "answer": "yes"},
  {"type": "choice", "id": "q2", "option": "<option value>"},
  {"type": "score",  "id": "q3", "level": 2}
]}
```

`answers` holds exactly one answer for every question of the decision,
each of its question's type and naming it by `id`:

| question | answer | counted as |
|---|---|---|
| `noul` | `answer`: `"yes"` or `"no"` | the noul 1 or 0 |
| `choice` | `option`: the `value` of one option | probability 1 for that option, 0 for the rest |
| `score` | `level`: the number of one level, the first being 0 | probability 1 for that level, 0 for the rest |

One pick per question: a vote is never split or ranked.

The result is `{}`: the vote is in. Otherwise the call fails, and says
why:

- The vote does not answer the questions (one unanswered or answered
  twice, an answer of the wrong type, an option or level the question
  does not have). The decision stays open, and the vote can be made
  again.
- The caller has no open decision by that id. It was never the
  caller's, its `structured_decisions` call has ended, or the caller
  has voted on it already. A decision is voted on once.

## Permissions

Declared at registration:

```json
[
  {"agents": {"actions": ["message"], "within": "any"}},
  {"daemons": {"actions": ["relay"], "within": "any"}}
]
```

The tool takes no arguments and is one tool for every agent that
declares it, so it cannot know at registration which agents a call
will list, and each grant reaches `"any"`.

- `message` over agents reaches the agents on this daemon: the
  structured decision agents asked, and the voters told.
- `relay` over remote daemons reaches the agents on others. This
  daemon judges only that the tool may relay to the first daemon of a
  route, and the daemon the route ends at judges the message, as the
  account this daemon's record names. `relay` is one action for every
  use of a remote daemon, so it allows any request relayed there, not
  messages only.

## State

The voters' queues are kept in the tool's own database, which the
daemon gives every container. A decision is there only while its
`structured_decisions` call runs, and whatever a previous run of the
tool left is deleted when the tool first uses the database. The tool
takes no secret.
