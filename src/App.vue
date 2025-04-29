<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import logo from './assets/logo.svg';

const error = ref("");  // Error message
const error_status = ref(false);  // Error status
const os_detail = ref(''); // OS details
const core_current_version = ref(''); // Core current version

const do_exit = async () => {
  await invoke("tg_stop");
}

const setupListeners = async () => {
  await listen("js_exit", do_exit);
};

async function checkReady(targetUrl) {
  for(let i=0; i<30; i++) {

    try {
      const res=await invoke("check_status")
      console.log("checkReadycheckReadycheckReadycheckReadycheckReadycheckReady",res);
      if (res){
        location.replace(targetUrl);
        return;
      }
    } catch (e) {
      console.log("checkReadychec",e);
    }
    await new Promise(r=>setTimeout(r, 2000));
  }
}

onMounted(async () => {
  console.log("Component mounted");
  await setupListeners();
  console.log("Starting initialization command")
  let start_port;
  try{
    const result = await invoke("tg_init");
    os_detail.value=result.os_detail
    core_current_version.value=result.core_current_version
    start_port=result.start_port
    let url=`http://127.0.0.1:${start_port}/?fr=ta&os=${os_detail.value}&cv=${core_current_version.value}`;
    await invoke("tg_start")
    await checkReady(url);
  }catch (e) {
    error_status.value=true
    error.value="Program initialization failed, please exit and restart!"
  }
});
</script>

<template>
  <div class="flex flex-col h-screen bg-gray-100">
    <!-- 主内容区域 -->
    <div class="flex-1 bg-white shadow-md overflow-auto relative">
      <!-- 加载中UI - 无论何时都显示此UI，除非有错误 -->
      <div class="w-full h-full flex items-center justify-center bg-gradient-to-b from-blue-50 to-white bg-gradient-animate">
        <div class="text-center w-[600px] p-12 bg-white/80 backdrop-blur-lg rounded-3xl shadow-[0_20px_60px_-10px_rgba(59,130,246,0.3)] border-2 border-blue-100/70 transition-all duration-500 animate-card-appear" style="max-height: 650px;">
          <!-- 添加Logo -->
          <div class="mb-4 animate-fade-in" style="animation-delay: 0.1s">
            <img :src="logo" alt="Logo" class="h-20 mx-auto drop-shadow-lg filter contrast-125 animate-logo-glow" />
          </div>
          
          <div v-if="!error_status" class="mb-10 relative">
            <!-- 大型光晕效果 -->
            <div class="absolute inset-0 rounded-full bg-blue-200/40 blur-2xl transform scale-150 animate-pulse-slow"></div>
            
            <!-- 外圆 -->
            <div class="w-40 h-40 mx-auto rounded-full bg-blue-50/90 flex items-center justify-center p-2 shadow-xl shadow-blue-200/50 relative border border-blue-100/50">
              <!-- 内圆及动画 -->
              <svg class="animate-spin h-24 w-24 text-blue-500 mx-auto" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                <path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            
            <!-- 装饰元素 -->
            <div class="absolute top-0 right-0 -mr-6 -mt-6 w-12 h-12 rounded-full bg-gradient-to-br from-blue-400 to-blue-500 opacity-70 animate-float"></div>
            <div class="absolute bottom-0 left-0 -ml-5 -mb-5 w-10 h-10 rounded-full bg-gradient-to-tr from-blue-300 to-indigo-400 opacity-60 animate-float" style="animation-delay: 0.5s"></div>
            <div class="absolute top-1/2 left-0 -ml-8 mt-3 w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 opacity-50 animate-float" style="animation-delay: 0.8s"></div>
            <div class="absolute bottom-1/3 right-0 -mr-7 mb-10 w-9 h-9 rounded-full bg-gradient-to-l from-indigo-400 to-blue-400 opacity-60 animate-float" style="animation-delay: 1.2s"></div>
            <div class="absolute top-1/4 right-1/4 w-6 h-6 rounded-full bg-gradient-to-tl from-sky-400 to-blue-300 opacity-50 animate-float" style="animation-delay: 1.5s"></div>
            <div class="absolute bottom-1/4 left-1/4 w-7 h-7 rounded-full bg-gradient-to-bl from-indigo-300 to-blue-400 opacity-40 animate-float" style="animation-delay: 0.9s"></div>
          </div>
          
          <!-- 错误状态显示 -->
          <div v-if="error_status" class="mb-10 text-center">
            <div class="mb-8 p-6 bg-red-50 rounded-xl text-base text-red-500 border-l-4 border-red-400 shadow-md max-w-lg mx-auto animate-fade-in">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 inline mr-2 mb-1 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              {{ error }}
            </div>
            
            <!-- 退出按钮 -->
            <button
              @click="do_exit" 
              class="relative overflow-hidden bg-gradient-to-r from-red-500 via-red-600 to-red-700 hover:from-red-600 hover:via-red-700 hover:to-red-800 text-white px-8 py-4 rounded-xl text-base font-medium transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1 flex items-center group mx-auto"
            >
              <span class="absolute inset-0 bg-white/10 transform scale-x-0 group-hover:scale-x-100 transition-transform origin-left duration-300"></span>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
              Exit Program
            </button>
          </div>
          
          <!-- 正常加载状态 -->
          <div v-if="!error_status">
            <h3 class="text-3xl font-bold text-gray-800 mb-6 animate-fade-in tracking-wide" style="animation-delay: 0.3s">TG-FF Starting</h3>
            
            <!-- 优雅的分隔线 -->
            <div class="w-24 h-1 mx-auto mb-6 rounded-full bg-gradient-to-r from-blue-300 to-indigo-400 opacity-70 animate-fade-in" style="animation-delay: 0.35s"></div>
            
            <p class="text-lg text-gray-600 max-w-lg mx-auto mb-10 leading-relaxed animate-fade-in" style="animation-delay: 0.4s">Please wait a moment, we are preparing amazing content for you...</p>
            
            <!-- 高级加载进度条 -->
            <div class="mt-8 max-w-lg mx-auto animate-fade-in" style="animation-delay: 0.5s">
              <div class="w-full h-3 bg-gray-100 rounded-full overflow-hidden shadow-inner">
                <div class="h-full loading-progress-bar"></div>
              </div>
              <!-- 加载指示器动画 -->
              <div class="flex justify-center space-x-2 mt-8">
                <div class="loading-dot" style="animation-delay: 0s"></div>
                <div class="loading-dot" style="animation-delay: 0.2s"></div>
                <div class="loading-dot" style="animation-delay: 0.4s"></div>
                <div class="loading-dot" style="animation-delay: 0.6s"></div>
                <div class="loading-dot" style="animation-delay: 0.8s"></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
}

body {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
}

button {
  cursor: pointer;
  transition: all 0.3s;
}

button:disabled {
  cursor: not-allowed;
}

input {
  font-family: inherit;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1.2s linear infinite;
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.animate-pulse-slow {
  animation: pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.6;
  }
  50% {
    opacity: 0.3;
  }
}

/* 卡片出现动画 */
.animate-card-appear {
  animation: cardAppear 1s cubic-bezier(0.19, 1, 0.22, 1) forwards;
}

@keyframes cardAppear {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.95);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 淡入动画 */
.animate-fade-in {
  animation: fadeIn 0.8s ease-in-out forwards;
  opacity: 0;
}

@keyframes fadeIn {
  0% {
    opacity: 0;
    transform: translateY(15px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 悬浮动画 */
.animate-float {
  animation: float 4s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-12px);
  }
}

/* 进度条加载动画 */
.loading-progress-bar {
  width: 30%;
  animation: loading 3s ease-in-out infinite;
  background-image: linear-gradient(to right, #3b82f6, #6366f1, #4f46e5, #6366f1, #3b82f6);
  background-size: 300% 100%;
}

@keyframes loading {
  0% {
    width: 0%;
    background-position: 100% 50%;
  }
  50% {
    width: 75%;
    background-position: 0% 50%;
  }
  100% {
    width: 98%;
    background-position: 100% 50%;
  }
}

/* 渐变动画 */
.bg-gradient-animate {
  background-size: 400% 400%;
  animation: gradient 15s ease infinite;
  background-image: linear-gradient(135deg, 
    rgba(239, 246, 255, 0.95), 
    rgba(219, 234, 254, 0.9), 
    rgba(191, 219, 254, 0.85), 
    rgba(147, 197, 253, 0.8),
    rgba(96, 165, 250, 0.75),
    rgba(59, 130, 246, 0.7),
    rgba(37, 99, 235, 0.65),
    rgba(59, 130, 246, 0.7),
    rgba(96, 165, 250, 0.75),
    rgba(147, 197, 253, 0.8),
    rgba(191, 219, 254, 0.85),
    rgba(219, 234, 254, 0.9),
    rgba(239, 246, 255, 0.95)
  );
}

@keyframes gradient {
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
}

/* 加载点动画 */
.loading-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: #3b82f6;
  animation: loadingDot 1.4s ease-in-out infinite;
}

@keyframes loadingDot {
  0%, 100% {
    transform: translateY(0);
    opacity: 0.3;
  }
  50% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

@media (prefers-color-scheme: dark) {
  .bg-gray-100 {
    background-color: #1a1a1a;
  }
  
  .bg-white {
    background-color: #2d2d2d;
  }
}

/* Logo动画效果 */
.animate-logo-glow {
  animation: logoGlow 2.5s ease-in-out infinite;
  transform-origin: center;
}

@keyframes logoGlow {
  0%, 100% {
    filter: drop-shadow(0 0 8px rgba(59, 130, 246, 0.3));
    transform: scale(1);
  }
  50% {
    filter: drop-shadow(0 0 15px rgba(59, 130, 246, 0.5));
    transform: scale(1.05);
  }
}
</style>
