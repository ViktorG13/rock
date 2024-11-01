const sendBtn = document.querySelector("#send");
const message = document.querySelector("#message");
const messages = document.querySelector("#messages");
let socket;
let reconnectInterval;
let isReconnecting = false; // Flag para controlar o estado de reconexão

function initWebSocket() {
  socket = new WebSocket("ws://localhost:3000");

  // Funções de evento do WebSocket
  socket.onopen = () => {
    updateStatus("Connected");
    clearInterval(reconnectInterval); // Limpar o intervalo de reconexão
    isReconnecting = false; // Atualizar flag para indicar que a conexão foi restabelecida
  };

  socket.onerror = () => updateStatus("Error...");
  
  socket.onclose = () => {
    updateStatus("Disconnected...");
    if (!isReconnecting) {
      startReconnect(); // Iniciar tentativa de reconexão se ainda não estiver tentando
    }
  };

  socket.onmessage = (msg) => messageHandler(msg);
}

function startReconnect() {
  isReconnecting = true; // Definir flag para evitar reconexões múltiplas
  updateStatus("Reconnecting...");
  
  reconnectInterval = setInterval(() => {
    if (socket.readyState === WebSocket.CLOSED) { // Tentar reconectar apenas se o WebSocket estiver realmente fechado
      initWebSocket();
    }
  }, 2000); // Tentar reconectar a cada 2 segundos
}

function updateStatus(message) {
  document.querySelector("p").innerText = message;
}

sendBtn.addEventListener("click", () => {
  if (socket.readyState === WebSocket.OPEN) {
    socket.send(message.value);
    message.value = ""; // Limpar o campo de entrada após o envio
  } else {
    updateStatus("Not connected, please wait...");
  }
});

function messageHandler(msg) {
  const newMessage = document.createElement("p");
  newMessage.innerText = msg.data;
  messages.appendChild(newMessage);
}

// Inicializar o WebSocket na primeira vez
initWebSocket();
