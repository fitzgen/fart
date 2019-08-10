const theme = document.getElementById("theme");

function updateTheme() {
  document.body.classList.remove("light");
  document.body.classList.remove("dark");
  const data = new FormData(theme);
  document.body.classList.add(data.get("theme"));
}

for (const input of theme.querySelectorAll("input")) {
  input.addEventListener("input", updateTheme);
}

updateTheme();

const userConstsForm = document.getElementById("user-consts");

function rerun() {
  const data = new FormData(userConstsForm);
  const consts = {};

  for (const [name, val] of data.entries()) {
    if (val !== "") {
      consts[name] = val;
    }
  }

  fetch("/rerun", {
    method: "POST",
    cache: "no-cache",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(consts)
  });
}

function debounce(f) {
  let id = null;
  return function debounced(...args) {
    clearTimeout(id);
    id = setTimeout(() => f.apply(this, args), 100);
  };
}

const debouncedRerun = debounce(rerun);

const regenerate = document.getElementById("regenerate");

regenerate.addEventListener("click", event => {
  event.preventDefault();
  rerun();
});

class UserConst {
  constructor(name, ty, value) {
    this.name = name;
    this.ty = ty;
    this.value = value;
    this.used = true;
    this.element = document.createElement("div");
    this.label = document.createElement("label");
    this.input = document.createElement("input");
    this.onInput = this.onInput.bind(this);

    this.element.className = "hbox user-const";

    this.update(name, ty, value);
    this.element.appendChild(this.label);

    this.input.addEventListener("input", this.onInput);
    this.input.setAttribute("type", "text");
    this.element.appendChild(this.input);
  }

  onInput(event) {
    event.preventDefault();
    debouncedRerun();
  }

  update(name, ty, value) {
    this.name = name;
    this.ty = ty;
    this.value = value;
    this.used = true;
    this.label.textContent = `${name}: ${ty} =`;
    this.input.setAttribute("name", name);
    this.input.setAttribute("placeholder", value);
  }

  destroy() {
    this.input.removeEventListener("input", this.onInput);
  }
}

class UserConstSet {
  constructor(container) {
    this.container = container;
    this.consts = new Map;
  }

  insert(name, ty, value) {
    let c = this.consts.get(name);
    if (c == null) {
      c = new UserConst(name, ty, value);
      this.container.appendChild(c.element);
      this.consts.set(name, c);
    } else {
      c.update(name, ty, value);
    }
  }

  sweep() {
    const newConsts = new Map;

    for (const [name, c] of this.consts) {
      if (c.used) {
        c.used = false;
        newConsts.add(name, c);
      } else {
        this.container.removeChild(c.element);
        c.destroy();
      }
    }

    this.consts = newConsts;
  }
}

const logs = document.getElementById("logs");
const latest = document.querySelector("#latest > object");
const events = new EventSource("/events");
const userConsts = new UserConstSet(userConstsForm);

events.addEventListener("start", _ => logs.textContent = "");
events.addEventListener("output", e => {
  const data = JSON.parse(e.data);
  for (const [_, name, ty, value] of data.matchAll(/.*fart: const ([\w_]+): ([\w_]+) = (.+);.*/g)) {
    userConsts.insert(name, ty, value);
  }
  logs.textContent += data;
});
events.addEventListener("finish", _ => {
  latest.setAttribute("data", `./images/latest.svg?${Date.now()}-${Math.random()}`);
});
events.onerror = event => {
  logs.textContent = `Error: disconnected from ${window.location.host}/events.`;
  console.error(event);
  regenerate.setAttribute("disabled", "");
};
