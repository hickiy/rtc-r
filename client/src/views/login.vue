<!-- hickey 2025/05/12 -->
<template>
  <el-form :model="loginForm" :rules="rules" ref="loginFormRef" label-width="80px" class="login-form">
    <el-form-item label="用户名" prop="username">
      <el-input v-model="loginForm.username" autocomplete="off" />
    </el-form-item>
    <el-form-item label="密码" prop="password">
      <el-input v-model="loginForm.password" type="password" autocomplete="off" />
    </el-form-item>
    <el-form-item>
      <el-button type="primary" @click="handleLogin" :loading="loading">登录</el-button>
    </el-form-item>
    <el-alert v-if="errorMsg" :title="errorMsg" type="error" show-icon class="mt-2" />
  </el-form>
</template>

<script>
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'

export default {
  name: 'Login',
  data() {
    return {
      loginForm: {
        username: '',
        password: ''
      },
      loading: false,
      errorMsg: '',
      rules: {
        username: [
          { required: true, message: '请输入用户名', trigger: 'blur' }
        ],
        password: [
          { required: true, message: '请输入密码', trigger: 'blur' }
        ]
      }
    }
  },
  methods: {
    handleLogin() {
      this.errorMsg = ''
      this.$refs.loginFormRef.validate(async (valid) => {
        if (!valid) return
        this.loading = true
        try {
          // 假设 WebSocket 服务地址为 ws://localhost:8080
          const ws = new WebSocket('ws://localhost:8080')
          ws.onopen = () => {
            ws.send(JSON.stringify({
              action: 'login',
              username: this.loginForm.username,
              password: this.loginForm.password
            }))
          }
          ws.onmessage = (event) => {
            const data = JSON.parse(event.data)
            if (data.success) {
              ElMessage.success('登录成功')
              // 登录成功后的逻辑
              // this.$router.push('/home')
            } else {
              this.errorMsg = data.message || '登录失败'
            }
            this.loading = false
            ws.close()
          }
          ws.onerror = () => {
            this.errorMsg = 'WebSocket 连接失败'
            this.loading = false
          }
        } catch (e) {
          this.errorMsg = '登录异常'
          this.loading = false
        }
      })
    }
  }
}
</script>

<style scoped>
.login-form {
  max-width: 350px;
  margin: 100px auto;
  padding: 40px 30px 30px 30px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0,0,0,0.1);
}
.mt-2 {
  margin-top: 16px;
}
</style>