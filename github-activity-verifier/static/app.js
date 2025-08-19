// GitHub Activity Verifier - Frontend JavaScript

class GitHubVerifier {
    constructor() {
        this.currentState = 'form'; // 'form' | 'loading' | 'result' | 'error'
        this.bindEvents();
        this.initFromURL();
    }
    
    bindEvents() {
        // フォーム送信
        document.getElementById('verify-btn').addEventListener('click', (e) => {
            e.preventDefault();
            this.handleVerification();
        });
        
        // リンクコピー
        document.getElementById('copy-link-btn').addEventListener('click', () => {
            this.copyShareLink();
        });
        
        // 新しい検証
        document.getElementById('new-verification-btn').addEventListener('click', () => {
            this.showForm();
        });
        
        // リトライボタン
        document.getElementById('retry-btn').addEventListener('click', () => {
            this.showForm();
        });
        
        // Enter keyでフォーム送信
        document.addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && this.currentState === 'form') {
                this.handleVerification();
            }
        });
        
        // ラジオボタンの変更でthresholdのplaceholderを更新
        document.querySelectorAll('input[name="verification-type"]').forEach(radio => {
            radio.addEventListener('change', () => {
                this.updateThresholdPlaceholder();
                this.updateVerificationLabels();
            });
        });
        
        // threshold入力の変更で表示を更新
        document.getElementById('threshold').addEventListener('input', () => {
            this.updateVerificationLabels();
        });
    }
    
    initFromURL() {
        const params = new URLSearchParams(window.location.search);
        const proofHash = params.get('proof');
        
        if (proofHash && /^[a-f0-9]{64}$/.test(proofHash)) {
            this.loadProofDetails(proofHash);
        } else {
            this.showForm();
        }
        
        this.updateThresholdPlaceholder();
        this.updateVerificationLabels();
    }
    
    updateThresholdPlaceholder() {
        const selectedType = document.querySelector('input[name="verification-type"]:checked').value;
        const thresholdInput = document.getElementById('threshold');
        
        const defaults = {
            'yearly_commits': '365',
            'consecutive_days': '100',
            'total_stars': '1000',
            'public_repos': '10'
        };
        
        thresholdInput.placeholder = `Default: ${defaults[selectedType]}`;
    }
    
    updateVerificationLabels() {
        const selectedType = document.querySelector('input[name="verification-type"]:checked').value;
        const thresholdInput = document.getElementById('threshold');
        const customThreshold = thresholdInput.value;
        
        const defaults = {
            'yearly_commits': 365,
            'consecutive_days': 100,
            'total_stars': 1000,
            'public_repos': 10
        };
        
        const typeLabels = {
            'yearly_commits': 'Commits/Year',
            'consecutive_days': 'Days Streak',
            'total_stars': 'Total Stars',
            'public_repos': 'Public Repos'
        };
        
        // 使用する閾値を決定
        const threshold = customThreshold ? parseInt(customThreshold) : defaults[selectedType];
        
        // ラジオボタンのラベルを更新
        document.querySelectorAll('input[name="verification-type"]').forEach(radio => {
            const span = radio.nextElementSibling;
            const radioType = radio.value;
            const radioThreshold = (radioType === selectedType && customThreshold) ? 
                parseInt(customThreshold) : defaults[radioType];
            
            span.textContent = `${radioThreshold}+ ${typeLabels[radioType]}`;
        });
    }
    
    async handleVerification() {
        const formData = this.getFormData();
        
        if (!this.validateForm(formData)) {
            return;
        }
        
        this.showLoading();
        
        try {
            const response = await fetch('/api/verify', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(formData)
            });
            
            const result = await response.json();
            
            if (!response.ok) {
                throw new Error(result.error || 'Verification failed');
            }
            
            this.handleVerificationSuccess(result);
            
        } catch (error) {
            console.error('Verification error:', error);
            this.showError(error.message);
        }
    }
    
    getFormData() {
        const username = document.getElementById('github-username').value.trim();
        const verificationType = document.querySelector('input[name="verification-type"]:checked').value;
        const thresholdInput = document.getElementById('threshold').value;
        
        const formData = {
            github_username: username,
            verification_type: verificationType
        };
        
        if (thresholdInput) {
            formData.threshold = parseInt(thresholdInput);
        }
        
        return formData;
    }
    
    validateForm(formData) {
        // GitHub username validation
        const usernameRegex = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,37}[a-zA-Z0-9])?$/;
        if (!formData.github_username) {
            this.showError('Please enter a GitHub username');
            return false;
        }
        
        if (!usernameRegex.test(formData.github_username)) {
            this.showError('Invalid GitHub username format. Must contain only alphanumeric characters and hyphens, cannot start or end with hyphen.');
            return false;
        }
        
        if (formData.github_username.includes('--')) {
            this.showError('GitHub username cannot contain consecutive hyphens');
            return false;
        }
        
        // Threshold validation
        if (formData.threshold && (formData.threshold < 1 || formData.threshold > 10000)) {
            this.showError('Threshold must be between 1 and 10000');
            return false;
        }
        
        return true;
    }
    
    async loadProofDetails(proofHash) {
        this.showLoading();
        
        try {
            const response = await fetch(`/proof/${proofHash}`);
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Proof not found');
            }
            
            const proof = await response.json();
            this.displayProofCertificate(proof);
            
        } catch (error) {
            console.error('Load proof error:', error);
            this.showError('Failed to load proof: ' + error.message);
        }
    }
    
    handleVerificationSuccess(result) {
        // URLを更新
        const newURL = `${window.location.pathname}?proof=${result.proof_hash}`;
        window.history.pushState(null, null, newURL);
        
        // 証明書表示
        this.displayProofCertificate(result);
    }
    
    displayProofCertificate(proof) {
        // 基本情報
        const usernameElement = document.getElementById('proof-username');
        const criteriaElement = document.getElementById('proof-criteria');
        const timestampElement = document.getElementById('proof-timestamp');
        
        if (usernameElement) usernameElement.textContent = proof.username;
        if (criteriaElement) criteriaElement.textContent = this.formatVerificationType(proof.verification_type, proof.threshold);
        if (timestampElement) timestampElement.textContent = this.formatDateTime(proof.verified_at);
        // Proof Hash表示（成功時のみ）
        const proofHashElement = document.getElementById('proof-hash');
        if (proofHashElement) {
            if (proof.proof_hash) {
                proofHashElement.textContent = proof.proof_hash;
                proofHashElement.parentElement.style.display = 'flex';
            } else {
                proofHashElement.parentElement.style.display = 'none';
            }
        }
        
        // 結果表示
        const resultElement = document.getElementById('proof-result');
        const statusBadge = document.getElementById('verification-status');
        
        if (resultElement && statusBadge) {
            if (proof.meets_criteria) {
                resultElement.textContent = '✅ VERIFIED';
                resultElement.style.color = '#155724';
                statusBadge.textContent = '✅ VERIFIED';
                statusBadge.className = 'status-badge verified';
            } else {
                resultElement.textContent = '❌ NOT VERIFIED';
                resultElement.style.color = '#721c24';
                statusBadge.textContent = '❌ NOT VERIFIED';
                statusBadge.className = 'status-badge failed';
            }
        }
        
        // TEE Attestation Tabs（成功時のみ表示）
        setTimeout(() => {
            this.displayTEEAttestation(proof);
        }, 100);
        
        // 共有リンク設定（成功時のみ）
        const shareSection = document.querySelector('.share-section');
        const shareLinkElement = document.getElementById('share-link');
        
        if (shareSection && shareLinkElement) {
            if (proof.proof_hash && proof.meets_criteria) {
                const shareURL = `${window.location.origin}/?proof=${proof.proof_hash}`;
                shareLinkElement.value = shareURL;
                shareSection.style.display = 'block';
            } else {
                shareSection.style.display = 'none';
            }
        }
        
        this.showProofDetails();
    }
    
    formatVerificationType(type, threshold = null) {
        const typeLabels = {
            'yearly_commits': 'Commits/Year',
            'consecutive_days': 'Days Streak',
            'total_stars': 'Total Stars',
            'public_repos': 'Public Repos'
        };
        
        const defaults = {
            'yearly_commits': 365,
            'consecutive_days': 100,
            'total_stars': 1000,
            'public_repos': 10
        };
        
        const actualThreshold = threshold || defaults[type];
        return `${actualThreshold}+ ${typeLabels[type]}`;
    }
    
    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('ja-JP', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            timeZone: 'UTC',
            timeZoneName: 'short'
        });
    }
    
    copyShareLink() {
        const linkInput = document.getElementById('share-link');
        linkInput.select();
        
        // Modern clipboard API
        if (navigator.clipboard) {
            navigator.clipboard.writeText(linkInput.value).then(() => {
                this.showCopyFeedback();
            }).catch(() => {
                // Fallback to execCommand
                document.execCommand('copy');
                this.showCopyFeedback();
            });
        } else {
            // Fallback for older browsers
            document.execCommand('copy');
            this.showCopyFeedback();
        }
    }
    
    showCopyFeedback() {
        const button = document.getElementById('copy-link-btn');
        const originalText = button.textContent;
        button.textContent = '✅ Copied!';
        button.style.background = '#28a745';
        
        setTimeout(() => {
            button.textContent = originalText;
            button.style.background = '';
        }, 2000);
    }
    
    // State管理メソッド
    showForm() {
        this.currentState = 'form';
        this.hideAll();
        document.getElementById('verification-form').style.display = 'block';
        
        // フォームリセット
        document.getElementById('github-username').value = '';
        document.getElementById('threshold').value = '';
        document.querySelector('input[name="verification-type"][value="yearly_commits"]').checked = true;
        this.updateThresholdPlaceholder();
        this.updateVerificationLabels();
        
        // URL更新
        window.history.pushState(null, null, window.location.pathname);
    }
    
    showLoading() {
        this.currentState = 'loading';
        this.hideAll();
        document.getElementById('loading').style.display = 'block';
        
        // ボタンの状態更新
        const verifyBtn = document.getElementById('verify-btn');
        verifyBtn.querySelector('.btn-text').style.display = 'none';
        verifyBtn.querySelector('.btn-spinner').style.display = 'inline';
        verifyBtn.disabled = true;
    }
    
    showProofDetails() {
        this.currentState = 'result';
        this.hideAll();
        document.getElementById('proof-details').style.display = 'block';
    }
    
    showError(message) {
        this.currentState = 'error';
        this.hideAll();
        document.getElementById('error').style.display = 'block';
        document.querySelector('#error .error-message').textContent = message;
        
        // ボタンの状態リセット
        this.resetVerifyButton();
    }
    
    hideAll() {
        ['verification-form', 'proof-details', 'loading', 'error'].forEach(id => {
            document.getElementById(id).style.display = 'none';
        });
        
        this.resetVerifyButton();
    }
    
    resetVerifyButton() {
        const verifyBtn = document.getElementById('verify-btn');
        verifyBtn.querySelector('.btn-text').style.display = 'inline';
        verifyBtn.querySelector('.btn-spinner').style.display = 'none';
        verifyBtn.disabled = false;
    }
    
    // TEE Attestation表示
    displayTEEAttestation(proof) {
        const teeSection = document.getElementById('tee-section');
        if (!teeSection) {
            console.error('Element tee-section not found');
            return;
        }
        
        if (proof.attestation_token) {
            teeSection.style.display = 'block';
            
            // Token Status & Raw Token
            this.displayTokenStatus(proof.attestation_token);
            this.displayRawToken(proof.attestation_token);
            
            // Claims表示
            if (proof.attestation_claims) {
                this.displayTokenClaims(proof.attestation_claims);
            }
            
            // Raw JSON表示
            this.displayRawJSON(proof);
            
            // タブ機能初期化
            this.initializeTabs();
        } else {
            teeSection.style.display = 'none';
        }
    }
    
    displayTokenStatus(token) {
        const statusElement = document.getElementById('token-status');
        if (!statusElement) {
            console.error('Element token-status not found');
            return;
        }
        
        if (token === 'MAA_NOT_CONFIGURED') {
            statusElement.innerHTML = '⚠️ MAA endpoint not configured';
            statusElement.className = 'token-status warning';
        } else if (token === 'MAA_UNAVAILABLE') {
            statusElement.innerHTML = '❌ MAA attestation service unavailable';
            statusElement.className = 'token-status error';
        } else {
            statusElement.innerHTML = '✅ MAA attestation token successfully retrieved';
            statusElement.className = 'token-status success';
        }
    }
    
    displayRawToken(token) {
        const rawElement = document.getElementById('token-raw');
        if (!rawElement) {
            console.error('Element token-raw not found');
            return;
        }
        rawElement.textContent = token;
    }
    
    displayTokenClaims(claims) {
        const gridElement = document.getElementById('claims-grid');
        if (!gridElement) {
            console.error('Element claims-grid not found');
            return;
        }
        gridElement.innerHTML = '';
        
        // 重要なclaimsを順序よく表示
        const importantClaims = [
            { key: 'iss', label: 'Issuer (MAA Instance)' },
            { key: 'x-ms-attestation-type', label: 'Attestation Type' },
            { key: 'x-ms-compliance-status', label: 'Compliance Status' },
            { key: 'iat', label: 'Issued At', format: 'timestamp' },
            { key: 'exp', label: 'Expires At', format: 'timestamp' },
            { key: 'jku', label: 'JWK Set URL (Public Keys)' },
            { key: 'kid', label: 'Key ID' },
            { key: 'x-ms-policy-hash', label: 'Policy Hash' },
            { key: 'x-ms-runtime', label: 'Runtime Data', format: 'json' }
        ];
        
        importantClaims.forEach(({ key, label, format }) => {
            if (claims[key] !== undefined) {
                const claimItem = document.createElement('div');
                claimItem.className = 'claim-item';
                
                const claimLabel = document.createElement('div');
                claimLabel.className = 'claim-label';
                claimLabel.textContent = label;
                
                const claimValue = document.createElement('div');
                claimValue.className = 'claim-value';
                
                if (format === 'timestamp') {
                    const date = new Date(claims[key] * 1000);
                    claimValue.textContent = date.toISOString() + ' (' + claims[key] + ')';
                } else if (format === 'json') {
                    claimValue.textContent = JSON.stringify(claims[key], null, 2);
                } else {
                    claimValue.textContent = claims[key];
                }
                
                claimItem.appendChild(claimLabel);
                claimItem.appendChild(claimValue);
                gridElement.appendChild(claimItem);
            }
        });
        
        // その他のclaims
        const otherClaims = Object.keys(claims).filter(key => 
            !importantClaims.some(important => important.key === key)
        );
        
        if (otherClaims.length > 0) {
            const otherHeader = document.createElement('div');
            otherHeader.style.fontWeight = 'bold';
            otherHeader.style.marginTop = '20px';
            otherHeader.style.marginBottom = '10px';
            otherHeader.textContent = 'Other Claims:';
            gridElement.appendChild(otherHeader);
            
            otherClaims.forEach(key => {
                const claimItem = document.createElement('div');
                claimItem.className = 'claim-item';
                
                const claimLabel = document.createElement('div');
                claimLabel.className = 'claim-label';
                claimLabel.textContent = key;
                
                const claimValue = document.createElement('div');
                claimValue.className = 'claim-value';
                claimValue.textContent = typeof claims[key] === 'object' ? 
                    JSON.stringify(claims[key], null, 2) : claims[key];
                
                claimItem.appendChild(claimLabel);
                claimItem.appendChild(claimValue);
                gridElement.appendChild(claimItem);
            });
        }
    }
    
    displayRawJSON(proof) {
        const rawElement = document.getElementById('raw-json');
        if (!rawElement) {
            console.error('Element raw-json not found');
            return;
        }
        rawElement.textContent = JSON.stringify(proof, null, 2);
    }
    
    initializeTabs() {
        const tabButtons = document.querySelectorAll('.tab-btn');
        const tabContents = document.querySelectorAll('.tab-content');
        
        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const targetTab = button.getAttribute('data-tab');
                
                // すべてのタブボタンからactiveクラスを除去
                tabButtons.forEach(btn => btn.classList.remove('active'));
                // すべてのタブコンテンツからactiveクラスを除去
                tabContents.forEach(content => content.classList.remove('active'));
                
                // クリックされたタブボタンにactiveクラスを追加
                button.classList.add('active');
                // 対応するタブコンテンツにactiveクラスを追加
                document.getElementById(targetTab + '-tab').classList.add('active');
            });
        });
    }
}

// アプリケーション初期化
document.addEventListener('DOMContentLoaded', () => {
    console.log('GitHub Activity Verifier initialized');
    window.verifier = new GitHubVerifier();
});