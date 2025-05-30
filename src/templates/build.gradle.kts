plugins {{
    java
}}

group = "com.example"
version = "0.1.0-SNAPSHOT"

repositories {{
    mavenCentral()
}}

dependencies {{
    implementation("{}")

    testImplementation(platform("org.junit:junit-bom:5.12.2"))
    testImplementation("org.junit.jupiter:junit-jupiter")
}}

tasks.test {{
    useJUnitPlatform()
}}
