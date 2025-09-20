const moveSnd = new Audio("/mp3/move.mp3");
let socket;

let state = {}, hovering = [], chat = [];
let playerLoaded = false;
let sessionId = localStorage.getItem('chess_session_id');

const getSquare = (sq) => $('#chessboard .square-' + sq);
const isDark = (sq) => sq.hasClass('black-3c85d');
const restart = () => location.reload();
const htmlEscape = (d) => d.replace(/</g, "&lt;").replace(/>/g, "&gt;");

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    socket = new WebSocket(wsUrl);
    
    socket.onopen = () => {
        console.log("connected");
        if (sessionId) {
            console.log("Attempting to reconnect with session:", sessionId);
            socket.send(JSON.stringify({ session_id: sessionId }));
        }
    };
    
    socket.onclose = () => {
        if (!state || state.status === "running") {
            chat.push({ name: "System", msg: "Connection lost. Refresh to reconnect to your game." });
            updateChat();
        }
    };
    
    socket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        console.log('Received:', message);

        if (message.type === 'state') {
            handleStateUpdate(message.data);
        } else if (message.type === 'chat') {
            handleChatMessage(message.data);
        } else if (message.session_id) {
            handleSessionResponse(message);
        }
    };
}

function handleSessionResponse(msg) {
    console.log("Session response:", msg);
    if (msg.game_found) {
        sessionId = msg.session_id;
        localStorage.setItem('chess_session_id', sessionId);
        console.log("Session established:", sessionId);
    } else {
        console.log("Session not found, will create new on upload");
        localStorage.removeItem('chess_session_id');
        sessionId = null;
    }
}

function handleChatMessage(msg) {
    chat.push(msg);
    chat = chat.slice(-15);
    updateChat();
}

function updateChat() {
    $("#chat").html(chat.map(c => `${htmlEscape(c.name)}: ${htmlEscape(c.msg)}`).join("<br />"));
}

function handleStateUpdate(data) {
    state = data;
    board.position(state.pos, true);
    moveSnd.play();
    $(".navbar").attr("class", "navbar navbar-expand-md bg-primary");
    $("#turn").text(`Turn ${state.turn_counter} / 50`);
    $(".row-5277c > div").css("background", "");

    if (data.status !== "running") {
        setTimeout(() => {
            $(".navbar").attr("class", "navbar navbar-expand-md bg-info");
            if (data.status === "draw") {
                Swal.fire({
                    title: 'Draw!',
                    icon: 'info'
                });
                $("#turn").text(`Draw! (Turn ${state.turn_counter})`);
            }
            else if (data.status === "turn limit") {
                Swal.fire({
                    title: 'Out of turns!',
                    icon: 'info'
                });
                $("#turn").text(`Out of turns! (Turn ${state.turn_counter})`);
            }
            else {
                const isWin = state.status === "win";
                const title = isWin && state.flag ? 'You Win!' : 'Checkmate!';
                const text = isWin && state.flag ? `Congratulations! Here's your flag: ${state.flag}` : '';
                
                Swal.fire({
                    title,
                    text,
                    icon: 'success',
                    allowOutsideClick: false,
                    allowEscapeKey: false,
                    showConfirmButton: true
                });
                $("#turn").text(`Checkmate - ${isWin ? 'You win' : 'You lose!'} (Turn ${state.turn_counter})`);
            }
        }, 500);
    }
}

const board = Chessboard('chessboard', {
    draggable: false,
    position: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
});

async function uploadPlayer() {
    const fileInput = document.getElementById('wasmFile');
    
    if (!fileInput.files.length) {
        showStatus('Please select a WASM file', 'danger');
        return;
    }
    
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        showStatus('WebSocket not connected', 'danger');
        return;
    }
    
    try {
        const file = fileInput.files[0];
        const arrayBuffer = await file.arrayBuffer();
        
        showStatus('Sending WASM file...', 'info');
        socket.send(arrayBuffer);
        
        playerLoaded = true;
        showStatus('WASM file sent successfully', 'success');
    } catch (error) {
        showStatus('Upload error: ' + error.message, 'danger');
    }
};

function showStatus(message, type) {
    const statusDiv = document.getElementById('playerStatus');
    statusDiv.className = `alert alert-${type}`;
    statusDiv.textContent = message;
    statusDiv.style.display = 'block';
}

connectWebSocket();