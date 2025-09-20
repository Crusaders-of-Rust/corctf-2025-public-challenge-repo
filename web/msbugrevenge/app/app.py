from flask import Flask, request, redirect, render_template, session, g, abort

import sqlite3
import os
import re
import secrets

try:
    from secret import FLAG, ADMIN_USERNAME, ADMIN_PASSWORD
except (ImportError, AttributeError):
    FLAG = 'corctf{EXAMPLE_FLAG}'
    ADMIN_USERNAME = 'admin'
    ADMIN_PASSWORD = 'password'

app = Flask(__name__)
app.secret_key = os.urandom(32)

DATABASE = 'msbug.db'

def get_db():
    if 'db' not in g:
        g.db = sqlite3.connect(DATABASE)
        g.db.row_factory = sqlite3.Row
    return g.db

@app.teardown_appcontext
def close_db(exception=None):
    db = g.pop('db', None)
    if db is not None:
        db.close()

def init_db():
    with app.app_context():
        db = get_db()
        db.execute('''
            CREATE TABLE IF NOT EXISTS reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                name TEXT NOT NULL,
                handled BOOLEAN NOT NULL DEFAULT 0
            )
        ''')

        existing = db.execute('SELECT COUNT(*) FROM reports').fetchone()[0]
        if existing == 0:
            db.execute(
                'INSERT INTO reports (content, name, handled) VALUES (?, ?, ?)',
                (FLAG, "jazz", 1)
            )
        db.commit()


@app.before_request
def add_nonce():
    if not getattr(request, 'csp_nonce', None):
        request.csp_nonce = secrets.token_hex(16)[:16]

@app.after_request
def set_security_headers(response):
    response.headers['X-Content-Type-Options'] = 'nosniff'
    response.headers['X-Frame-Options'] = 'SAMEORIGIN'
    response.headers['X-XSS-Protection'] = '1; mode=block'
    response.headers['Referrer-Policy'] = 'strict-origin-when-cross-origin'
    return response

@app.route('/', methods=['GET', 'POST'])
def index():
    error = None
    if request.method == 'POST':
        content = request.form.get('content', '').strip()
        name = request.form.get('name').strip()
        if len(content) == 0 or len(content) > 512:
            error = 'Bug report must be between 1 and 512 characters. (120 was a bit short for some reports D:)'
        if len(name) == 0 or len(name) > 120:
            error = 'Submitting a name is now required due to abuse.'
        if web_application_firewall(content) or web_application_firewall(name):
            error = 'Please dont try to hack us :C'
        else:
            db = get_db()
            db.execute('INSERT INTO reports (content, name, handled) VALUES (?, ?, ?)', (content, name, 0))
            db.commit()
            return redirect('/')
    return render_template('index.html', error=error)


@app.route('/login', methods=['GET', 'POST'])
def login():
    error = None
    if request.method == 'POST':
        username = request.form.get('username', '')
        password = request.form.get('password', '')
        if username == ADMIN_USERNAME and password == ADMIN_PASSWORD:
            session['logged_in'] = True
            return redirect('/admin')
        else:
            error = 'Invalid credentials.'
    return render_template('login.html', error=error)


@app.route('/logout')
def logout():
    session.pop('logged_in', None)
    return redirect('/')


@app.route('/admin')
def admin():
    if not session.get('logged_in'):
        return redirect('/login')
    db = get_db()
    reports = db.execute('SELECT id, content AS safe_content, name AS safe_name, handled FROM reports ORDER BY id DESC').fetchall()
    return render_template('admin.html', reports=reports, csp_nonce=request.csp_nonce)


@app.route('/handle/<int:report_id>', methods=['POST'])
def handle_report(report_id):
    if not session.get('logged_in'):
        return redirect('/login')

    password = request.form.get('password')

    if password != ADMIN_PASSWORD:
        return abort(403, description="Invalid password")

    db = get_db()
    db.execute('UPDATE reports SET handled = 1 WHERE id = ?', (report_id,))
    db.commit()
    return redirect('/admin')



# Matches: any opening/closing of risky tags OR inline on* event handlers
_WAF_HIT = re.compile(
    r"</?\s*(?:iframe|frame|frameset|object|embed|applet|param|svg|foreignObject|"
    r"animate|set|animateMotion|animateTransform|math|link|style|img|video|audio|"
    r"source|track|picture|input|button|select|option|portal)\b"
    r"|\bon[a-z0-9_-]*\s*=",
    re.IGNORECASE,
)

def web_application_firewall(content: str) -> bool:
    """True if content contains dangerous HTML tags or inline event handlers; otherwise False."""
    return bool(_WAF_HIT.search(content or ""))


if __name__ == '__main__':
    if not os.path.exists(DATABASE):
        init_db()
    app.run()
else:
    with app.app_context():
        if not os.path.exists(DATABASE):
            init_db()
