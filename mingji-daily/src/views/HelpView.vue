<template>
  <div class="help">
    <h1>使用帮助</h1>
    <p class="sub">第一次用？跟着「快速上手」走一遍，5 分钟就能玩转。</p>

    <details open class="sec">
      <summary>🚀 快速上手（3 步）</summary>
      <ol class="steps">
        <li><b>记一笔</b>：左侧「记一笔」→ 选支出/收入 → 输金额 → 点分类 → 选账户 → 保存</li>
        <li><b>看明细</b>：左侧「明细」→ 本月账单按天分组，点击任意一条可修改/删除</li>
        <li><b>看统计</b>：左侧「统计」→ 自动按自然月汇总，点分类可跳转到对应明细</li>
      </ol>
      <p class="tip">💡 保存后会记住你上次用的分类和账户，日常记账 5 秒搞定。</p>
    </details>

    <details class="sec">
      <summary>✏️ 记一笔</summary>
      <ul class="list">
        <li>金额支持小数（如 25.5）；输入后按 <b>Enter</b> 或点「保存」</li>
        <li>分类分两级：先选大类（餐饮/交通…），再选小类（早餐/地铁…）；没有小类的大类直接点选</li>
        <li>账户可选现金/微信/支付宝/银行卡，也可在「设置 → 账户管理」添加自己的</li>
        <li>日期默认今天，可改历史日期；备注选填</li>
        <li>可附加照片/视频（见下方「照片与视频」）</li>
      </ul>
    </details>

    <details class="sec">
      <summary>📋 明细</summary>
      <ul class="list">
        <li>按天分组，每天头部显示当日支出/收入小计</li>
        <li>顶部筛选：收支类型 / 分类 / 账户 / 月份（可勾选"不限时间"）/ 备注搜索</li>
        <li>点击账单弹出编辑框：改金额、分类、账户、日期、备注、附件，也可删除</li>
        <li>删除账单会连同其照片/视频一起删除（有二次确认）</li>
      </ul>
    </details>

    <details class="sec">
      <summary>📊 统计与自定义周期（核心功能）</summary>
      <p class="sub">统计页包含：总收入/支出/结余/日均、分类占比图（可切一二级）、每日趋势、与上一周期对比。</p>
      <p class="sub"><b>想按自己的节奏统计？</b>去「设置 → 周期管理」新建规则：</p>
      <table class="tbl">
        <tr><th>类型</th><th>怎么设</th><th>例子</th></tr>
        <tr><td>按月</td><td>选每月几号开始（1~28 日）</td><td>每月 25 日发薪 → 统计 25 号~次月 24 号</td></tr>
        <tr><td>按周</td><td>选周几开始</td><td>每周一起始</td></tr>
        <tr><td>自定义</td><td>长度 + 单位（天/周/月）</td><td>每 14 天 / 每 3 周 / 每 2 个月</td></tr>
      </table>
      <ul class="list">
        <li>统计页顶部可切换规则，用 ◀ ▶ 翻看历史周期，「回到本期」一键归位</li>
        <li>规则只影响统计口径，删除规则不影响账单数据</li>
      </ul>
    </details>

    <details class="sec">
      <summary>📥 导入以前的账单</summary>
      <p class="sub">两种方式：粘贴文本，或选择 txt / md / docx 文件（.doc 请先另存为 .docx）。</p>
      <p class="sub">支持这样的文本（每行一条，日期/项目/金额）：</p>
      <pre class="code">2025-01-15 午餐 25元
1月16日 地铁 4元 交通
2025-01-17 收入 工资 12000元
昨天和朋友聚餐 128</pre>
      <p class="sub">也支持一行多笔的格式：</p>
      <pre class="code">9月1日:早饭5，午饭10，晚饭16，总计</pre>
      <ul class="list">
        <li>解析后逐条核对（可改日期/金额/分类），<b>标红的行</b>需要补全并勾选才会导入</li>
        <li>识别失败的格式，用「自定义解析规则」：选分隔符 + 选每列含义即可，不用懂技术</li>
        <li>导入后 24 小时内可在页面底部「撤销」</li>
      </ul>
    </details>

    <details class="sec">
      <summary>📷 照片与视频</summary>
      <ul class="list">
        <li>记一笔或编辑账单时点「📎 添加照片 / 视频」</li>
        <li>单笔最多 <b>9 张照片 + 1 个视频</b>；超限会整批拦截，删除多余的再添加</li>
        <li>照片自动生成缩略图，点击可全屏查看；视频可直接播放</li>
        <li>文件会复制到数据目录，删除原文件不影响查看；删除账单会连带删除附件</li>
      </ul>
    </details>

    <details class="sec">
      <summary>💾 数据安全</summary>
      <ul class="list">
        <li>每次退出自动备份数据库（保留最近 7 份）</li>
        <li>「数据管理」→ 立即备份（可选含照片/视频）→ 需要时点「恢复」（恢复后重启生效）</li>
        <li>导出 CSV（Excel 打开）/ JSON（全部数据留档）</li>
        <li>数据只保存在本机：便携版在程序同目录 <code>data\</code>，安装版在 <code>AppData\Roaming\铭记日常\</code>；整个文件夹拷走即可搬家</li>
      </ul>
    </details>

    <details class="sec">
      <summary>❓ 常见问题</summary>
      <table class="tbl">
        <tr><th>问题</th><th>解答</th></tr>
        <tr><td>双击提示"已保护你的电脑"</td><td>软件暂未商业签名，点「更多信息 → 仍要运行」</td></tr>
        <tr><td>双击没反应</td><td>需 Win10 21H2+ / Win11；老系统请装微软 WebView2 运行时</td></tr>
        <tr><td>怎么卸载</td><td>便携版删文件夹；安装版在「设置 → 应用」卸载（数据默认保留）</td></tr>
        <tr><td>数据会上传吗</td><td>不会，默认完全离线，账单只在你的电脑上</td></tr>
        <tr><td>分类删不掉</td><td>被账单使用或含子分类的分类只能「停用」（防误删保护）</td></tr>
        <tr><td>记错了怎么办</td><td>明细页编辑或删除；批量导入的可 24 小时内整批撤销</td></tr>
      </table>
    </details>
  </div>
</template>

<style scoped>
.help {
  max-width: 760px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.help h1 {
  font-size: 24px;
}

.sub {
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.7;
}

.sec {
  background: var(--card);
  border-radius: 12px;
  padding: 4px 16px;
}

.sec summary {
  cursor: pointer;
  padding: 12px 0;
  font-weight: 600;
  font-size: 14px;
  list-style: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sec summary::after {
  content: "▾";
  color: var(--text-3);
  transition: transform 0.2s;
}

.sec:not([open]) summary::after {
  transform: rotate(-90deg);
}

.sec summary::-webkit-details-marker {
  display: none;
}

.sec[open] {
  padding-bottom: 14px;
}

.steps,
.list {
  padding: 0 0 6px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.steps li,
.list li {
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.7;
}

.tip {
  font-size: 12px;
  color: var(--text-3);
  padding-bottom: 6px;
}

.code {
  background: var(--bg);
  border-radius: 8px;
  padding: 10px 12px;
  font-family: Consolas, monospace;
  font-size: 12px;
  color: var(--text-1);
  line-height: 1.8;
  margin: 4px 0 8px;
  overflow-x: auto;
}

.tbl {
  width: 100%;
  border-collapse: collapse;
  margin: 6px 0 10px;
  font-size: 12px;
}

.tbl th,
.tbl td {
  border: 1px solid var(--line);
  padding: 7px 10px;
  text-align: left;
  color: var(--text-2);
  line-height: 1.6;
}

.tbl th {
  background: var(--bg);
  color: var(--text-1);
  font-weight: 600;
}

code {
  background: var(--bg);
  padding: 1px 5px;
  border-radius: 4px;
  font-family: Consolas, monospace;
  font-size: 12px;
}
</style>
