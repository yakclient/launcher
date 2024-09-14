import styles from "./console.module.sass"
import {Button} from "react-bootstrap";
import {emit, listen} from '@tauri-apps/api/event'
import {ReactElement, useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {useRouter} from "next/router";

export type ConsoleLine = {
    is_err: string,
    frag: number[]
}

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

    useEffect(() => {
        listen<ConsoleLine>('process-stdout', (event) => {
            setLines((prev) => [...prev, event.payload])
        }).then((it) => {
        })
    }, [])

    useEffect(() => {
        if (consoleContentRef.current) {
            consoleContentRef.current.scrollTop = consoleContentRef.current.scrollHeight;
        }
    }, [lines])

    return <>
        <div id={styles.consoleHeader}>
            <h1>Game output</h1>
        </div>
        <div id={styles.consoleContent} ref={consoleContentRef}>
            {lines.map((line, index) =>
                <span key={index} className={styles.consoleLine} style={{
                    color: line.is_err ? "red" : "inherit"
                }}>
                    {mapLine(String.fromCharCode(...line.frag))}
                </span>
            )}
        </div>
        <div id={styles.end}>
            <Button
                onClick={() => {
                    invoke("end_launch_process").then(() => {
                        router.push("/home")
                    })
                }}
                variant={"outline-danger"}
            >End Process</Button>
        </div>
    </>
}

export default Console