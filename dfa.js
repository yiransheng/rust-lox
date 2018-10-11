const FINAL = 1;
const FAIL = 2;

class DFA {
  constructor() {
    this.states = [0, FINAL, FAIL];
    this.transitions = [];
  }

  _nextStateID() {
    return this.states.length;
  }

  _addState() {
    const id = this._nextStateID();
    this.states.push(id);
    return id;
  }

  addWord(word) {
    if (!word.length) {
      throw Error("Empty word");
    }
    this._addWord(word, 0);
  }

  _addWord(word, startState) {
    if (!word.length) {
      return;
    }
    const a = word.charAt(0);
    const rest = word.slice(1);

    let transition = this.transitions.find(([s, t, ns]) => {
      return s === startState && t === a;
    });

    if (!transition) {
      const nextState = rest.length ? this._addState() : FINAL;
      transition = [startState, a, nextState];
      this.transitions.push(transition);
    }

    this._addWord(rest, transition[2]);
  }

  printRustCode(printer) {
    this.printMatchState(printer);
    for (const state of this.states) {
      if (state !== FINAL && state !== FAIL) {
        this.printState(state, printer);
      }
    }
  }

  printMatchState(printer) {
    printer.line("fn consume(&mut self, t: char) -> bool {");
    printer.block();
    printer.line("let next_state = match self.state {");
    printer.block();
    for (const state of this.states) {
      if (state === FINAL || state === FAIL) {
        printer.line(`${state} => { return false },`);
      } else {
        printer.line(`${state} => _state_${state}(t),`);
      }
    }
    printer.line("_ => unreachable!(),");
    printer.blockEnd();
    printer.line("};");
    printer.line("self.state = next_state;")
    printer.line("true");
    printer.blockEnd();
    printer.line("}");
  }

  printState(state, printer) {
    printer.line("#[inline(always)]");
    printer.line(`fn _state_${state} (t: char) -> u8 {`);
    printer.block();
    printer.line("match t {");
    printer.block();
    for (const [s, t, ns] of this.transitions) {
      if (s === state) {
        printer.line(`'${t}' => ${ns},`);
      }
    }
    printer.line(`_ => ${FAIL},`);
    printer.blockEnd();
    printer.line("}");
    printer.blockEnd();
    printer.line("}");
  }
}

class PrintBlock {
  constructor(indent, parent) {
    this._indent = indent;
    this._parent = parent;
  }
  line(line) {
    if (this._indent > 0) {
      const spaces = " ".repeat(this._indent * 4);
      console.log(`${spaces}${line}`);
    } else {
      console.log(line);
    }

    return this;
  }
  block() {
    return new PrintBlock(this._indent + 1, this);
  }
  blockEnd() {
    return this._parent || new PrintBlock(0);
  }
}

function Printer() {
  let printer = new PrintBlock(0);

  return {
    block() {
      printer = printer.block();
    },
    line(line) {
      printer.line(line);
    },
    blockEnd() {
      printer = printer.blockEnd();
    }
  };
}

const words = [
  "and",
  "class",
  "else",
  "false",
  "for",
  "fun",
  "if",
  "nil",
  "or",
  "print",
  "return",
  "super",
  "this",
  "true",
  "var",
  "while"
];

const dfa = new DFA();

for (const word of words) {
  dfa.addWord(word);
}

dfa.printRustCode(Printer());
