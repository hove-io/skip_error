node {
    stage ('Checkout') {
        checkout scm
    }
    try {
        stage ('Quality checks') {
            parallel 'format': {
                sh 'make format'
            }, 'clippy': {
                sh 'make lint'
            }
        }
        stage ('Tests') {
            sh 'make test'
        }
    } catch (err) {
        echo "Caught: ${err}"
            currentBuild.result = 'FAILURE'
    } finally {
        stage ('Cleanup') {
            cleanWs()
        }
    }
}
