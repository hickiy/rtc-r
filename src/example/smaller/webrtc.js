const startButton = document.getElementById('startButton');
const callButton = document.getElementById('callButton');
const hangupButton = document.getElementById('hangupButton');
const localVideo = document.getElementById('localVideo');
const remoteVideo = document.getElementById('remoteVideo');

let localStream;
let pc1;
let pc2;

startButton.onclick = start;
callButton.onclick = call;
hangupButton.onclick = hangup;

async function start() {
  const videoFile = './video.mp4'; // 替换为本地 MP4 文件的路径
  localVideo.src = videoFile;
  localVideo.play();
}

async function call() {
  const configuration = {};
  pc1 = new RTCPeerConnection(configuration);
  pc2 = new RTCPeerConnection(configuration);

  pc1.onicecandidate = e => pc2.addIceCandidate(e.candidate);
  pc2.onicecandidate = e => pc1.addIceCandidate(e.candidate);

  pc2.ontrack = e => remoteVideo.srcObject = e.streams[0];

  localStream = localVideo.captureStream(); // 从视频元素捕获流
  localStream.getTracks().forEach(track => pc1.addTrack(track, localStream));

  const offer = await pc1.createOffer();
  await pc1.setLocalDescription(offer);
  await pc2.setRemoteDescription(offer);

  const answer = await pc2.createAnswer();
  await pc2.setLocalDescription(answer);
  await pc1.setRemoteDescription(answer);
}

function hangup() {
  pc1.close();
  pc2.close();
  pc1 = null;
  pc2 = null;
}