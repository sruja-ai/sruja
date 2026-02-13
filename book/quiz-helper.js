/**
 * Interactive Quiz Helper for Sruja Courses (Markdown Checkboxes)
 * Handles checkbox selection (radio-style), answer checking, and feedback display
 */

(function () {
  "use strict";

  // Configuration
  const QUIZ_QUESTION_CLASS = "quiz-question";
  const CHECK_BUTTON_CLASS = "check-answer-btn";
  const FEEDBACK_CLASS = "answer-feedback";
  const EXPLANATION_CLASS = "explanation";

  // Initialize when DOM is ready
  function init() {
    const quizContainer = document.querySelector(".content");
    if (!quizContainer) return;

    // Find all quiz questions (blocks containing checkboxes)
    const questions = identifyQuizQuestions(quizContainer);

    // Setup each question
    questions.forEach(setupQuestion);

    // Observe for dynamically added quizzes
    observeQuizzes(quizContainer);
  }

  // Identify quiz questions by looking for blocks with checkboxes and check buttons
  function identifyQuizQuestions(container) {
    const questions = [];
    const checkButtons = container.querySelectorAll(`.${CHECK_BUTTON_CLASS}`);

    checkButtons.forEach((button) => {
      // Find the question block that contains this button
      let questionBlock = button.closest("ul, ol, p, div");

      // Walk up to find the actual question container
      while (questionBlock && !isQuestionBlock(questionBlock)) {
        questionBlock = questionBlock.parentElement;
      }

      if (questionBlock && !questions.includes(questionBlock)) {
        questions.push(questionBlock);
      }
    });

    return questions;
  }

  // Check if an element is a question block (contains checkboxes and check button)
  function isQuestionBlock(element) {
    const hasCheckboxes = element.querySelector('input[type="checkbox"]');
    const hasCheckButton = element.querySelector(`.${CHECK_BUTTON_CLASS}`);
    return hasCheckboxes && hasCheckButton;
  }

  // Setup a single question
  function setupQuestion(question) {
    const checkButton = question.querySelector(`.${CHECK_BUTTON_CLASS}`);
    if (!checkButton) return;

    const checkboxes = question.querySelectorAll('input[type="checkbox"]');
    const feedback = question.querySelector(`.${FEEDBACK_CLASS}`);
    const explanation = feedback?.querySelector(`.${EXPLANATION_CLASS}`);

    // Make checkboxes behave like radio buttons (only one per question)
    checkboxes.forEach((checkbox) => {
      checkbox.addEventListener("change", function (e) {
        if (e.target.checked) {
          // Uncheck all other checkboxes in this question
          checkboxes.forEach((cb) => {
            if (cb !== e.target) {
              cb.checked = false;
            }
          });
        }
      });
    });

    // Handle check button click
    checkButton.addEventListener("click", function () {
      // Get selected answer
      const selectedCheckbox = question.querySelector(
        'input[type="checkbox"]:checked',
      );

      if (!selectedCheckbox) {
        alert("Please select an answer before checking.");
        return;
      }

      // Find the option text (next to the checkbox)
      const selectedText = getOptionText(selectedCheckbox);
      const correctAnswer = checkButton.dataset.correct;

      // Disable all checkboxes
      checkboxes.forEach((cb) => (cb.disabled = true));

      // Show feedback
      if (feedback) {
        feedback.style.display = "block";
        const feedbackText = feedback.querySelector(".feedback-text");

        if (selectedText && selectedText.includes(correctAnswer + ")")) {
          // Correct!
          feedbackText.innerHTML =
            '<span class="feedback-correct">✓ Correct!</span>';
          selectedCheckbox.closest("li")?.classList.add("correct-option");
        } else {
          // Incorrect
          const correctCheckbox = findCorrectCheckbox(
            checkboxes,
            correctAnswer,
          );
          feedbackText.innerHTML = `<span class="feedback-incorrect">✗ Incorrect. The correct answer is ${correctAnswer.toUpperCase()}</span>`;
          selectedCheckbox.closest("li")?.classList.add("incorrect-option");
          if (correctCheckbox) {
            correctCheckbox.closest("li")?.classList.add("correct-option");
          }
        }

        // Show explanation
        if (explanation) {
          explanation.style.display = "block";
        }
      }

      // Disable check button
      checkButton.disabled = true;
      checkButton.textContent = "Answer Checked";

      // Check if all questions in this lesson are answered
      checkQuizCompletion(question);
    });
  }

  // Get the text next to a checkbox
  function getOptionText(checkbox) {
    // Try to get text from the next element
    const parent = checkbox.closest("li");
    if (parent) {
      return parent.textContent;
    }
    // Fallback: get parent element text
    return checkbox.parentElement?.textContent || "";
  }

  // Find the checkbox that corresponds to the correct answer
  function findCorrectCheckbox(checkboxes, correctAnswer) {
    for (const cb of checkboxes) {
      const text = getOptionText(cb);
      if (text.includes(correctAnswer + ")")) {
        return cb;
      }
    }
    return null;
  }

  // Check if all questions in the current page are answered correctly
  function checkQuizCompletion(answeredQuestion) {
    const allQuestions = document.querySelectorAll(`.${CHECK_BUTTON_CLASS}`);
    const totalQuestions = allQuestions.length;
    let correctCount = 0;

    allQuestions.forEach((btn) => {
      if (btn.disabled) {
        const correctAnswer = btn.dataset.correct;
        const selectedCheckbox = btn
          .closest("div")
          ?.querySelector('input[type="checkbox"]:checked');

        if (selectedCheckbox) {
          const selectedText = getOptionText(selectedCheckbox);
          if (selectedText && selectedText.includes(correctAnswer + ")")) {
            correctCount++;
          }
        }
      }
    });

    // If all questions are answered correctly, dispatch event for progress tracking
    if (correctCount === totalQuestions && totalQuestions > 0) {
      dispatchQuizCompletedEvent(correctCount, totalQuestions);
    }
  }

  // Dispatch custom event when quiz is completed
  function dispatchQuizCompletedEvent(score, total) {
    const event = new CustomEvent("quiz-completed", {
      detail: {
        score: score,
        total: total,
        percentage: (score / total) * 100,
      },
    });
    document.dispatchEvent(event);
  }

  // Observe for dynamically added quizzes
  function observeQuizzes(container) {
    const observer = new MutationObserver(function (mutations) {
      mutations.forEach(function (mutation) {
        if (mutation.addedNodes) {
          mutation.addedNodes.forEach(function (node) {
            if (node.nodeType === 1) {
              // Element node
              const checkButtons = node.querySelectorAll
                ? node.querySelectorAll(`.${CHECK_BUTTON_CLASS}`)
                : [];
              checkButtons.forEach((btn) => {
                const question = btn.closest("ul, ol, p, div");
                while (question && !isQuestionBlock(question)) {
                  question.parentElement;
                }
                if (question && isQuestionBlock(question)) {
                  setupQuestion(question);
                }
              });
            }
          });
        }
      });
    });

    observer.observe(container, {
      childList: true,
      subtree: true,
    });
  }

  // Initialize on DOM ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
