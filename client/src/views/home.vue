<!-- hickey 2025/05/12 -->
<template>
  <el-row style="height: 100vh;">
    <el-col :span="6" style="border-right: 1px solid #eee; padding: 20px;">
      <el-card>
        <div slot="header">会话列表</div>
        <el-list>
          <el-list-item
            v-for="user in users"
            :key="user.id"
            @click="selectUser(user)"
            :class="{ 'is-active': selectedUser && selectedUser.id === user.id }"
            style="cursor: pointer;"
          >
            {{ user.name }}
          </el-list-item>
        </el-list>
      </el-card>
    </el-col>
    <el-col :span="18" style="padding: 20px;">
      <el-card>
        <div slot="header">
          <span v-if="selectedUser">与 {{ selectedUser.name }} 的会话</span>
          <span v-else>请选择会话对象</span>
        </div>
        <div v-if="selectedUser" style="display: flex; gap: 20px;">
          <div>
            <div>本地视频</div>
            <video ref="localVideo" autoplay playsinline style="width: 320px; height: 240px; background: #000;"></video>
          </div>
          <div>
            <div>远端视频</div>
            <video ref="remoteVideo" autoplay playsinline style="width: 320px; height: 240px; background: #000;"></video>
          </div>
        </div>
      </el-card>
    </el-col>
  </el-row>
</template>

<script>
import { ElMessage } from 'element-plus'

export default {
  name: 'WebRTCHome',
  data() {
    return {
      users: [
        { id: 1, name: 'Alice' },
        { id: 2, name: 'Bob' },
        { id: 3, name: 'Charlie' }
      ],
      selectedUser: null,
      localStream: null,
      remoteStream: null,
      peerConnection: null
    }
  },
  methods: {
    async selectUser(user) {
      if (this.selectedUser && this.selectedUser.id === user.id) return
      this.selectedUser = user
      await this.startWebRTC()
    },
    async startWebRTC() {
      // 清理旧连接
      if (this.peerConnection) {
        this.peerConnection.close()
        this.peerConnection = null
      }
      if (this.localStream) {
        this.localStream.getTracks().forEach(track => track.stop())
        this.localStream = null
      }
      // 获取本地媒体流
      try {
        this.localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true })
        this.$refs.localVideo.srcObject = this.localStream
      } catch (e) {
        ElMessage.error('无法获取本地视频流')
        return
      }
      // 创建PeerConnection
      this.peerConnection = new RTCPeerConnection()
      this.peerConnection.ontrack = (event) => {
        if (!this.remoteStream) {
          this.remoteStream = new MediaStream()
          this.$refs.remoteVideo.srcObject = this.remoteStream
        }
        this.remoteStream.addTrack(event.track)
      }
      this.localStream.getTracks().forEach(track => {
        this.peerConnection.addTrack(track, this.localStream)
      })
      // 以下为简化演示，未实现信令服务
      // 实际应用需通过信令服务器交换SDP和ICE候选
    }
  },
  beforeDestroy() {
    if (this.peerConnection) this.peerConnection.close()
    if (this.localStream) this.localStream.getTracks().forEach(track => track.stop())
  }
}
</script>

<style scoped>
.is-active {
  background: #f0f9eb;
}
</style>