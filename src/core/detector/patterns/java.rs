use regex::Regex;

pub struct JavaPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl JavaPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Compiled bytecode
            Regex::new(r"\.class$").unwrap(),
            Regex::new(r"\.jar$").unwrap(),
            Regex::new(r"\.war$").unwrap(),
            Regex::new(r"\.ear$").unwrap(),
            Regex::new(r"\.aar$").unwrap(),
            
            // Build directories
            Regex::new(r"target/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"out/").unwrap(),
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"classes/").unwrap(),
            
            // Maven
            Regex::new(r"\.m2/").unwrap(),
            Regex::new(r"\.mvn/").unwrap(),
            
            // Gradle
            Regex::new(r"\.gradle/").unwrap(),
            Regex::new(r"gradle-app\.setting").unwrap(),
            Regex::new(r"gradlew").unwrap(),
            Regex::new(r"gradlew\.bat").unwrap(),
            
            // SBT (Scala Build Tool)
            Regex::new(r"\.sbt/").unwrap(),
            Regex::new(r"project/target/").unwrap(),
            
            // Ivy
            Regex::new(r"\.ivy2/").unwrap(),
            
            // Leiningen (Clojure)
            Regex::new(r"\.lein-").unwrap(),
            Regex::new(r"\.nrepl-port").unwrap(),
            
            // IDE generated files
            Regex::new(r"\.idea/").unwrap(),
            Regex::new(r"\.eclipse/").unwrap(),
            Regex::new(r"\.metadata/").unwrap(),
            Regex::new(r"\.settings/").unwrap(),
            Regex::new(r"\.project").unwrap(),
            Regex::new(r"\.classpath").unwrap(),
            
            // Android
            Regex::new(r"\.apk$").unwrap(),
            Regex::new(r"\.aab$").unwrap(),
            Regex::new(r"\.dex$").unwrap(),
            Regex::new(r"R\.java$").unwrap(),
            Regex::new(r"proguard/").unwrap(),
            
            // Spring Boot
            Regex::new(r"\.jar\.original$").unwrap(),
            
            // JVM crash dumps
            Regex::new(r"hs_err_pid").unwrap(),
            Regex::new(r"core\.\d+").unwrap(),
            
            // Native libraries
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.dll$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            Regex::new(r"\.jnilib$").unwrap(),
        ];

        let cache_patterns = vec![
            // Maven caches
            Regex::new(r"\.m2/repository").unwrap(),
            Regex::new(r"target/maven-").unwrap(),
            Regex::new(r"target/surefire-").unwrap(),
            Regex::new(r"target/failsafe-").unwrap(),
            
            // Gradle caches
            Regex::new(r"\.gradle/caches").unwrap(),
            Regex::new(r"\.gradle/wrapper").unwrap(),
            Regex::new(r"\.gradle/daemon").unwrap(),
            Regex::new(r"\.gradle/native").unwrap(),
            Regex::new(r"build/tmp/").unwrap(),
            
            // SBT caches
            Regex::new(r"\.sbt/").unwrap(),
            Regex::new(r"project/project/").unwrap(),
            Regex::new(r"project/target/").unwrap(),
            
            // Ivy caches
            Regex::new(r"\.ivy2/").unwrap(),
            
            // IDE caches
            Regex::new(r"\.idea/caches/").unwrap(),
            Regex::new(r"\.idea/libraries/").unwrap(),
            Regex::new(r"\.idea/modules\.xml").unwrap(),
            Regex::new(r"\.idea/compiler\.xml").unwrap(),
            Regex::new(r"\.idea/jarRepositories\.xml").unwrap(),
            
            // Testing caches
            Regex::new(r"target/test-").unwrap(),
            Regex::new(r"build/test-").unwrap(),
            Regex::new(r"\.jacoco").unwrap(),
            Regex::new(r"jacoco\.exec").unwrap(),
            
            // Spring Boot DevTools
            Regex::new(r"\.spring-boot-devtools-").unwrap(),
            
            // Kotlin compilation cache
            Regex::new(r"\.kotlin/").unwrap(),
            Regex::new(r"kotlin-build/").unwrap(),
            
            // Android caches
            Regex::new(r"\.android/").unwrap(),
            Regex::new(r"build/intermediates/").unwrap(),
            Regex::new(r"build/generated/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp").unwrap(),
            Regex::new(r"\.temp").unwrap(),
            Regex::new(r"\.log").unwrap(),
            
            // Documentation generation
            Regex::new(r"target/site/").unwrap(),
            Regex::new(r"build/docs/").unwrap(),
            
            // Profiling
            Regex::new(r"\.hprof").unwrap(),
            Regex::new(r"\.jfr").unwrap(),
            
            // Checkstyle
            Regex::new(r"\.checkstyle").unwrap(),
            Regex::new(r"checkstyle-").unwrap(),
            
            // PMD
            Regex::new(r"\.pmd").unwrap(),
            Regex::new(r"pmd\.xml").unwrap(),
            
            // SpotBugs
            Regex::new(r"\.spotbugs").unwrap(),
            Regex::new(r"spotbugsXml\.xml").unwrap(),
        ];

        let extensions = vec![
            // Java
            "java".to_string(),
            
            // Kotlin
            "kt".to_string(), "kts".to_string(),
            
            // Scala
            "scala".to_string(), "sc".to_string(),
            
            // Groovy
            "groovy".to_string(), "gvy".to_string(), "gy".to_string(), "gsh".to_string(),
            
            // Clojure
            "clj".to_string(), "cljs".to_string(), "cljc".to_string(), "edn".to_string(),
            
            // Build files
            "gradle".to_string(),
            
            // Configuration files
            "properties".to_string(),
            
            // Android
            "aidl".to_string(),
            
            // AspectJ
            "aj".to_string(),
            
            // Jython
            "jy".to_string(),
            
            // JRuby
            "jrb".to_string(),
        ];

        Self {
            external_patterns,
            cache_patterns,
            extensions,
        }
    }

    pub fn get_external_patterns(&self) -> &[Regex] {
        &self.external_patterns
    }

    pub fn get_cache_patterns(&self) -> &[Regex] {
        &self.cache_patterns
    }

    pub fn get_extensions(&self) -> &[String] {
        &self.extensions
    }

    pub fn get_script_names() -> Vec<&'static str> {
        vec![
            // Maven
            "pom.xml", "pom.properties", "settings.xml",
            "mvnw", "mvnw.cmd", ".mvn/wrapper/maven-wrapper.properties",
            
            // Gradle
            "build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts",
            "gradle.properties", "gradlew", "gradlew.bat",
            "gradle/wrapper/gradle-wrapper.properties",
            
            // SBT (Scala)
            "build.sbt", "project/build.properties", "project/plugins.sbt",
            "project/Dependencies.scala", "project/Settings.scala",
            
            // Leiningen (Clojure)
            "project.clj", "profiles.clj", "lein-env",
            
            // Boot (Clojure)
            "build.boot", "boot.properties",
            
            // Clojure CLI
            "deps.edn", "bb.edn",
            
            // Ant
            "build.xml", "ant.xml", "build.properties",
            
            // Ivy
            "ivy.xml", "ivysettings.xml",
            
            // Spring Boot
            "application.properties", "application.yml", "application.yaml",
            "application-dev.properties", "application-prod.properties",
            "bootstrap.properties", "bootstrap.yml",
            
            // Logging
            "logback.xml", "logback-spring.xml", "log4j.properties",
            "log4j2.xml", "log4j2.yml", "logging.properties",
            
            // Testing
            "testng.xml", "junit-platform.properties",
            
            // IDE configuration
            ".project", ".classpath", ".factorypath",
            "*.iml", "*.ipr", "*.iws",
            ".idea/", ".settings/", ".metadata/",
            
            // Android
            "AndroidManifest.xml", "build.gradle", "gradle.properties",
            "proguard-rules.pro", "proguard-android.txt",
            "local.properties", "keystore.properties",
            
            // Web applications
            "web.xml", "context.xml", "server.xml",
            "faces-config.xml", "beans.xml",
            
            // Persistence
            "persistence.xml", "hibernate.cfg.xml",
            "mybatis-config.xml", "ehcache.xml",
            
            // Security
            "security.xml", "shiro.ini", "jaas.conf",
            
            // CI/CD
            ".github/workflows/java.yml", ".github/workflows/maven.yml",
            ".github/workflows/gradle.yml", ".gitlab-ci.yml",
            "Jenkinsfile", "azure-pipelines.yml", "bitbucket-pipelines.yml",
            "circle.yml", ".circleci/config.yml", ".travis.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            ".dockerignore", "Dockerfile.jvm", "Dockerfile.native",
            
            // Deployment
            "Procfile", "app.yaml", "appengine-web.xml",
            "jboss-web.xml", "weblogic.xml", "glassfish-web.xml",
            
            // Code quality
            "checkstyle.xml", "pmd.xml", "spotbugs.xml",
            "findbugs.xml", "sonar-project.properties",
            
            // Documentation
            "README.md", "CHANGELOG.md", "CONTRIBUTING.md",
            "javadoc.xml", "overview.html",
            
            // License
            "LICENSE", "LICENSE.txt", "COPYING", "NOTICE",
            
            // Environment
            ".env", ".env.local", ".env.development", ".env.production",
            
            // Microservices
            "docker-compose.yml", "kubernetes.yml", "helm/",
            "service.yml", "deployment.yml",
            
            // Quarkus
            "quarkus.properties", "native-image.properties",
            
            // Micronaut
            "micronaut.properties", "bootstrap.yml",
            
            // Vert.x
            "vertx.json", "vertx-options.json",
            
            // JMX
            "jmx.properties", "management.properties",
            
            // JPA/Hibernate
            "orm.xml", "persistence.xml", "hibernate.properties",
            
            // Flyway/Liquibase
            "flyway.conf", "liquibase.properties",
            
            // Kafka
            "kafka.properties", "consumer.properties", "producer.properties",
            
            // Elasticsearch
            "elasticsearch.yml", "log4j2.properties",
            
            // Monitoring
            "micrometer.properties", "actuator.properties",
            
            // Build tools configuration
            "toolchains.xml", "extensions.xml", "lifecycle-mapping-metadata.xml",
            
            // Native compilation
            "native-image.properties", "reflection-config.json",
            "resource-config.json", "proxy-config.json",
            "jni-config.json", "serialization-config.json",
        ]
    }
} 