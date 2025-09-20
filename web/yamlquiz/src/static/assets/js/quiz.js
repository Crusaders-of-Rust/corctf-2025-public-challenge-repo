const $ = document.querySelector.bind(document);

const triesLeft = Number(localStorage.tries || "2");
if (triesLeft <= 0) {
  $('#gameover').style.display = 'block';
}
else {
  $('#remaining').innerText = triesLeft === 1 ? `1 try` : `${triesLeft} tries`;
  $('#start').style.display = 'block';
}

let num = 0;

function start() {
  localStorage.tries = String(triesLeft - 1);

  $('#quiz').innerHTML = generate();
  $('#start').style.display = 'none';
  $('#q0').style.display = 'block';
}

function next() {
  if (Array.from($('#q' + num).querySelectorAll('input'))
          .filter(i => i.checked)
          .length === 0) {
    // you didn't pick an option lmao
    return;
  }

  $('#q' + num).classList =
      'mt-4 question animate__animated animate__fadeOutLeft';
  $('#q' + num + ' button').onclick = null;
  num += 1;
  setTimeout(() => {
    $('#q' + num).style.display = 'block';
    $('#q' + num).classList =
        'mt-4 question animate__animated animate__fadeInRight';
  }, 500);
}

function finish() {
  $('#q' + num).classList =
      'mt-4 question animate__animated animate__fadeOutLeft';
  $('#q' + num + ' button').onclick = null;

  const responses =
      Array.from(document.querySelectorAll('.question'))
          .map(
              q => Array.from(q.children[2].querySelectorAll('input'))
                       .findIndex(a => a.checked));
  const score = responses.filter((r, i) => r == quiz[i].correct).length;
  setTimeout(() => {
    fetch('/submit', {
      method: 'POST',
      body: new URLSearchParams({
        'result': `
result:
  - score: ${score}
        `.trim()
      })
    })
        .then(r => r.json())
        .then(j => {
          if (j.pass) {
            $('#reward').innerText = j.flag;
            $('#pass').style.display = 'block';
          } else {
            $('#fail').style.display = 'block';
          }
        });
  }, 1250);
}

function generate() {
  let template = (q, num) => `
    <div class="mt-4 question" style="display: none" id="q${num}">
        <h4>Question ${num + 1}: What does this parse as?</h4>
        <h5>${marked.parse('```yaml\n' + q.question + '\n```')}</h5>
        <div>
            ${
      q.answers
          .map(
              (a, i) => `
            <div class="form-check">
                <input class="form-check-input" type="radio" id="q${num}-a${
                  i}" name="q${num}">
                <label class="form-check-label" for="q${num}-a${i}">${
                  marked.parse(a)}</label>
            </div>
            `)
          .join('\n')}
        </div>
        ${
      num !== quiz.length - 1 ?
          (`<button class="btn btn-primary mt-2" type="button" onclick="next()">Next →</button>`) :
          (`<button class="btn btn-info mt-2" type="button" onclick="finish()">Finish →</button>`)}
    </div>`;

  return quiz.map(template).join('\n');
}

// these questions shamelessly yoinked from https://www.ohyaml.wtf/
const quiz = [
  {
    'question': 'confirm: yes',
    'answers':
        ['confirm: yes', 'confirm: "yes"', 'confirm: true', 'confirm: on'],
    'correct': 2
  },
  {
    'question':
        'geoblock_regions:\n    - us #united states\n    - fr #france\n    - no #norway\n    - sf #san francisco\n    - in #india\n    - uk #united kingdom',
    'answers': [
      'The manifest is rejected with a YAML parse error, so the deployment never reaches Kubernetes.',
      'The Pod starts, but your app crashes on startup.',
      'Everything works as expected',
      'The pod starts and runs, but the readiness probe never succeeds'
    ],
    'correct': 1
  },
  {
    'question': 'permissions: 0755',
    'answers': [
      'permissions: 493', 'permissions: "0755"', 'permissions: 755',
      'permissions: 0o755'
    ],
    'correct': 0
  },
  {
    'question': 'code: 095',
    'answers': ['code: ', 'code: 77', 'code: "095"', 'code: 95'],
    'correct': 2
  },
  {
    'question': 'config_value: 010',
    'answers': [
      'config_value: 8', 'config_value: 10', 'config_value: "010"',
      'Parser error: leading zeros are not allowed'
    ],
    'correct': 1
  },
  {
    'question':
        'baseList: &fruits\n    - apple\n    - banana\n\nsalad:\n    - orange\n    - *fruits',
    'answers': [
      'salad:\n    - orange\n    - apple\n    - banana',
      'salad:\n  - orange\n  baseList:\n    - apple\n    - banana',
      'salad: ["orange", "apple", "banana"]',
      'salad: \n -  orange\n  - [apple, banana]'
    ],
    'correct': 3
  },
  {
    'question':
        'fruit: &f "apple"\nbasket:\n  - orange\n  - *f\n  - &f banana\n  - *f',
    'answers': [
      'basket:\n  - orange\n  - apple\n  - banana\n  - banana',
      'basket:\n  - orange\n  - apple\n  - banana\n  - apple',
      'basket:\n  - orange\n  - *f\n  - *f\n  - *f',
      'basket:\n  - orange\n  - apple\n  - apple\n  - apple'
    ],
    'correct': 0
  },
  {
    'question': 'port_mapping:\n  - 22:22\n  - 80:80\n  - 443:443',
    'answers': [
      '{"port_mapping": ["22:22", "80:80", "443:443"]}',
      '{"port_mapping": [1342, "80:80", "443:443"]}',
      '{"port_mapping": [1342, 4840, 26603]}', 'YAML parse error'
    ],
    'correct': 1
  },
  {
    'question':
        'durations:\n  - 1:10      \n  - 02:40    \n  - 1:        \n  - 0:0',
    'answers': [
      '[70, "02:40", "1:", "0:0"]', '[70, 160, "1:0", 0]',
      '[70, "02:40", "1:", 0]', 'YAML parse error'
    ],
    'correct': 2
  },
  {
    'question':
        'allow_postgres_versions:\n  - 9.5.25\n  - 9.6.24\n  - 10.23\n  - 12.13',
    'answers': [
      '{"allow_postgres_versions": \n        ["9.5.25", "9.6.24", "10.23", "12.13"]}',
      '{"allow_postgres_versions": \n        ["9.5.25", "9.6.24", 10.23, 12.13]}',
      '{"allow_postgres_versions": \n        [9.5.25, 9.6.24, 10.23, 12.13]}',
      'Parser error — "invalid version literals"'
    ],
    'correct': 1
  },
  {
    'question': 'key:',
    'answers': ['key: null', 'key: ""', 'key: {}', '# parse error'],
    'correct': 0
  },
  {
    'question': 'yes: true',
    'answers': ['true: true', '"yes": true', 'yes: true', 'Yes: True'],
    'correct': 0
  },
  {
    'question': 'message: >\n  Hello\n  World',
    'answers': [
      'message: "Hello World\\n"', 'message: "Hello\\nWorld"',
      'message: "Hello World"', 'message: "Hello\\nWorld\\n"'
    ],
    'correct': 0
  },
  {
    'question': 'config: |\n  username: admin\n  password: 1234',
    'answers': [
      'config: "username: admin\\npassword: 1234\\n"',
      'config:\n  username: admin\n  password: 1234',
      'config: "username: admin password: 1234"',
      'config: |\n  username: admin\n  password: 1234'
    ],
    'correct': 0
  },
  {
    'question':
        'settings:\n  mode: {{ default "auto" .Values.mode }}\n  replicas: {{ default 3 .Values.count }}',
    'answers': [
      'settings:\n  mode: auto\n  replicas: 3',
      'settings:\n  mode: ""\n  replicas: 0',
      'settings:\n  mode: auto\n  replicas: 0',
      'settings:\n  mode: "auto"\n  replicas: 3'
    ],
    'correct': 0
  },
  {
    'question':
        'debug: {{ if .Values.debug }} enabled {{ else }} \n         disabled {{ end }}',
    'answers':
        ['debug: enabled', 'debug: disabled', 'debug: false', 'debug: "false"'],
    'correct': 0
  },
  {
    'question': 'config:\n{{ .Values.extra | toYaml }}',
    'answers': [
      'config:\nkey: value', 'config:\n  key: value', 'config: key: value',
      'config:\n  key: value\n  config: null'
    ],
    'correct': 0
  },
  {
    'question':
        'metadata:\n  name: {{ required "Name is required!" .Values.name }}',
    'answers': [
      'The name field is omitted', 'Renders name: Name is required!',
      'Renders name: null',
      'Template rendering fails with error: "Name is required!"'
    ],
    'correct': 3
  },
  {
    'question': 'name: ""\ncount: 0',
    'answers': [
      'Renders name: guest, replicas: 1', 'Renders name: "", replicas: 1',
      'Error: "name is mandatory" (chart render fails)',
      'Renders name: guest, then fails on replicas (0 treated as empty)'
    ],
    'correct': 2
  },
  {
    'question':
        '# manifests-endpoints.yaml\nserve:\n  - /healthz\n  - *html          \n  - /index.html\n  - /favicon.ico\n  - *png',
    'answers': [
      'The manifest is accepted; serve becomes ["/healthz", "/index.html", "/index.html"]',
      'YAML parse error: unknown anchor \'html\' referenced',
      'The manifest is accepted; unresolved aliases are treated as literal strings like *html',
      'The manifest is accepted, but unresolved aliases are silently converted to null'
    ],
    'correct': 1
  },
  {
    'question': 'serve:\n  - /robots.txt\n  - /favicon.ico\n  - !.git',
    'answers': [
      'serve: ["/robots.txt", "/favicon.ico", ""]',
      'serve: ["/robots.txt", "/favicon.ico", "git"]',
      'serve: ["/robots.txt", "/favicon.ico"]',
      'ConstructorError: could not determine a constructor for the tag \'!.git\''
    ],
    'correct': 3
  }
];