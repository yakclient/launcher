import styles from "./console.module.sass"
import {Alert, Button} from "react-bootstrap";
import {useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {useRouter} from "next/router";
import {Alerts, ConsoleChannel, ConsoleLine, useConsole} from "@/pages/_app";


const Console: React.FC = () => {
    const [lines, setLines] = useState<ConsoleLine[]>([]);
    const router = useRouter();
    const consoleContentRef = useRef<HTMLDivElement>(null);

    const mapLine = (line: string): React.ReactNode => {
        let whitespace_mapped_line = line
            .replaceAll(' ', '\u00A0')         // Replace all spaces with non-breaking spaces
            .replaceAll('\t', '\u00A0\u00A0\u00A0\u00A0');
        return <>
            {
                Array.from(whitespace_mapped_line)
                    .map((str) => {
                        return str != "\n" ? <>{str}</> : <br/>
                    })
            }
        </>
    }

    const console = useConsole()

    if (console.channel) console.channel.onmessage = (message) => {
        setLines((prev) => [...prev, message])
    }

    useEffect(() => {
        if (consoleContentRef.current) {
            consoleContentRef.current.scrollTop = consoleContentRef.current.scrollHeight;
        }
    }, [lines])

    return <Alerts.Consumer>
        {addAlert =>
            <>
                <div id={styles.consoleHeader}>
                    <h1>Game output</h1>
                </div>
                <div id={styles.consoleContent} ref={consoleContentRef}>
                    {lines.map((line, index) =>
                        <span key={index} className={styles.consoleLine} style={{
                            color: line.is_err ? "red" : "inherit"
                        }}>{mapLine(String.fromCharCode(...line.frag))}</span>
                    )}
                </div>
                <div id={styles.end}>
                    <Button
                        onClick={() => {
                            router.push("/home")
                            invoke("end_launch_process").then(() => {
                            }).catch((it) => {
                                addAlert(
                                    "danger",
                                    <>
                                        <Alert.Heading>Error!</Alert.Heading>
                                        <hr/>
                                        {it.toString()}
                                    </>
                                )
                            })
                        }}
                        variant={"outline-danger"}
                    >End Process</Button>
                </div>
            </>
        }
    </Alerts.Consumer>
}

export default Console